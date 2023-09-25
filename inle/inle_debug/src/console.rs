use inle_cfg::{Cfg_Var, Config};
use inle_common::colors;
use inle_core::env::Env_Info;
use inle_gfx::render;
use inle_gfx::render_window::Render_Window_Handle;
use inle_input::bindings::Input_Action_Modifiers;
use inle_input::events::Input_Raw_Event;
use inle_input::input_state::Input_State;
use inle_input::keyboard;
use inle_math::rect::Rect;
use inle_math::vector::{Vec2f, Vec2u};
use inle_resources::gfx;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io;

mod history;

use history::{Direction, History};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Console_Status {
    Open,
    Closed,
}

const OUTPUT_SIZE: usize = 250;
const HIST_SIZE: usize = 50;
const COLOR_HISTORY: colors::Color = colors::rgba(200, 200, 200, 200);
const HIST_FILE: &str = ".console_hist.txt";

#[derive(Clone, Default)]
pub struct Console_Config {
    pub font: gfx::Font_Handle,

    pub font_size: Cfg_Var<u32>,
    pub ui_scale: Cfg_Var<f32>,
    pub pad_x: Cfg_Var<f32>,
    pub linesep: Cfg_Var<f32>,
    pub opacity: Cfg_Var<f32>,          // [0, 1]
    pub cur_line_opacity: Cfg_Var<f32>, // [0, 1]
}

pub struct Console {
    pub status: Console_Status,
    pub pos: Vec2u,
    pub size: Vec2u,
    // This should be (externally) set equal to the "toggle_console" action keys.
    pub toggle_console_keys: Vec<keyboard::Key>,
    pub cfg: Console_Config,

    cur_line: String,
    cur_pos: usize,

    enqueued_cmd: Option<String>,

    output: Vec<(String, colors::Color)>,
    history: History<String>,

    // { cur_cmd => hints relative to it } (empty string => all commands)
    // @Speed: we should avoid having more copies of the same hint set if more commands share it.
    pub hints: HashMap<String, Vec<String>>,
    hints_displayed: Vec<usize>,
    selected_hint: usize,
}

pub fn save_console_hist(console: &Console, env: &Env_Info) -> io::Result<()> {
    use std::io::{prelude::*, BufWriter};

    let mut path = env.working_dir.to_path_buf();
    path.push(HIST_FILE);

    let mut file = BufWriter::new(File::create(path)?);
    let mut latest_line = None;
    for line in &console.history {
        let line = line.trim();
        if !line.is_empty() && Some(line) != latest_line {
            writeln!(file, "{}", line)?;
            latest_line = Some(line);
        }
    }

    Ok(())
}

pub fn load_console_hist(console: &mut Console, env: &Env_Info) -> io::Result<()> {
    use std::io::{prelude::*, BufReader};

    let mut path = env.working_dir.to_path_buf();
    path.push(HIST_FILE);

    let mut file = BufReader::new(File::open(path)?);
    for _ in 0..HIST_SIZE {
        let mut line = String::new();
        match file.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) if !line.trim().is_empty() => console.history.push(line),
            Err(err) => {
                return Err(err);
            }
            _ => {}
        }
    }

    Ok(())
}

impl Console {
    pub fn new() -> Self {
        Self {
            cfg: Console_Config::default(),
            status: Console_Status::Closed,
            pos: Vec2u::default(),
            size: Vec2u::default(),
            cur_line: String::new(),
            cur_pos: 0,
            toggle_console_keys: vec![],
            enqueued_cmd: None,
            output: Vec::with_capacity(OUTPUT_SIZE),
            history: History::with_capacity(HIST_SIZE),
            hints: HashMap::new(),
            hints_displayed: vec![],
            selected_hint: 0,
        }
    }

    pub fn init(&mut self, cfg: Console_Config) {
        self.cfg = cfg;
    }

    pub fn toggle(&mut self) -> Console_Status {
        use Console_Status::*;
        self.status = match self.status {
            Open => Closed,
            Closed => Open,
        };
        self.status
    }

    pub fn update(&mut self, input_state: &Input_State) {
        for event in &input_state.raw.events {
            self.process_event(event, input_state.raw.kb_state.modifiers_pressed);
        }
    }

    pub fn pop_enqueued_cmd(&mut self) -> Option<String> {
        self.enqueued_cmd.take()
    }

    pub fn add_hints<I>(&mut self, cmd: &str, hints: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.hints
            .entry(cmd.to_string())
            .or_insert_with(Vec::default)
            .extend(hints);
    }

    pub fn output_line<T: ToString>(&mut self, line: T, color: colors::Color) {
        self.output.push((line.to_string(), color));
    }

    fn process_event(&mut self, event: &Input_Raw_Event, modifiers: Input_Action_Modifiers) {
        debug_assert!(self.cur_pos <= self.cur_line.len());

        let line_changed = match *event {
            Input_Raw_Event::Key_Pressed { code } => self.process_key(code, modifiers),
            Input_Raw_Event::Key_Repeated { code } => self.process_key(code, modifiers),
            _ => false,
        };

        if line_changed {
            self.update_hints();
        }
    }

    // Returns whether the line was changed or not.
    fn process_key(&mut self, code: keyboard::Key, modifiers: Input_Action_Modifiers) -> bool {
        use inle_input::bindings::modifiers::*;
        use keyboard::Key;

        let ctrl = (modifiers & MOD_CTRL) != 0;
        let shift = (modifiers & MOD_SHIFT) != 0;
        match code {
            _ if self.toggle_console_keys.contains(&code) => {
                self.status = Console_Status::Closed;
                false
            }

            Key::BackSpace => {
                if ctrl {
                    self.del_prev_word();
                } else {
                    self.del_prev_char();
                }

                true
            }

            Key::Up => {
                if self.hints_displayed.is_empty() {
                    if let Some(line) = self.history.move_and_read(Direction::To_Older) {
                        self.cur_line = line.to_string();
                        self.cur_pos = self.cur_line.len();
                    }
                } else if self.selected_hint == self.hints_displayed.len() - 1 {
                    self.selected_hint = 0;
                } else {
                    self.selected_hint += 1;
                }

                false
            }

            Key::Down => {
                if self.hints_displayed.is_empty() {
                    if !self.history.is_cursor_past_end() {
                        self.cur_line = self
                            .history
                            .move_and_read(Direction::To_Newer)
                            .map_or_else(|| String::from(""), |s| s.to_string());
                        self.cur_pos = self.cur_line.len();
                    }
                } else if self.selected_hint == 0 {
                    self.selected_hint = self.hints_displayed.len() - 1;
                } else {
                    self.selected_hint -= 1;
                }

                false
            }

            Key::Left => {
                if ctrl {
                    self.move_one_word(-1);
                } else {
                    self.move_one_char(-1);
                }

                false
            }

            Key::Right => {
                if ctrl {
                    self.move_one_word(1);
                } else {
                    self.move_one_char(1);
                }
                false
            }

            Key::A if ctrl => {
                self.cur_pos = 0;
                false
            }

            Key::Home => {
                self.cur_pos = 0;
                false
            }

            Key::E if ctrl => {
                self.cur_pos = self.cur_line.len();
                false
            }

            Key::End => {
                self.cur_pos = self.cur_line.len();
                false
            }

            Key::W if ctrl => {
                self.del_prev_word();
                true
            }

            Key::Delete => {
                if ctrl {
                    self.del_next_word();
                } else {
                    self.del_next_char();
                }
                true
            }

            Key::K if ctrl => {
                self.cur_line.truncate(self.cur_pos);
                true
            }

            Key::D if ctrl => {
                self.cur_line.clear();
                self.cur_pos = 0;
                true
            }

            Key::Return => {
                self.commit_line();
                true
            }

            Key::Tab => {
                if !self.hints_displayed.is_empty() {
                    // @Improve: this is a pretty rudimentary behaviour: consider improving.
                    let (cmd, _) = self.get_hint_key_and_rest().unwrap();
                    self.cur_line = cmd.to_string()
                        + if cmd.is_empty() { "" } else { " " }
                        + &self.hints[cmd][self.hints_displayed[self.selected_hint]]
                        + " ";
                    self.cur_pos = self.cur_line.len();
                }
                true
            }

            _ => {
                if let Some(c) = keyboard::key_to_char(code, shift) {
                    self.cur_line.insert(self.cur_pos, c);
                    self.cur_pos += 1;
                }
                true
            }
        }
    }

    fn commit_line(&mut self) {
        let cmdline = self.cur_line.trim().to_string();
        if cmdline.is_empty() {
            return;
        }
        self.history.push(cmdline.clone());
        self.output_line(cmdline.clone(), COLOR_HISTORY);
        self.enqueued_cmd = Some(cmdline);
        self.cur_line.clear();
        self.cur_pos = 0;
    }

    fn update_hints(&mut self) {
        self.hints_displayed.clear();

        let cur_line = self.cur_line.trim();
        if cur_line.is_empty() {
            return;
        }

        let cmd;
        let rest;
        {
            let cmd_and_rest = self.get_hint_key_and_rest().unwrap();
            cmd = cmd_and_rest.0.to_string();
            rest = cmd_and_rest.1.to_string();
        }

        if let Some(hints) = self.hints.get(&cmd) {
            for (i, hint) in hints.iter().enumerate() {
                if hint.contains(&rest) {
                    self.hints_displayed.push(i);
                }
            }
            // Sort alphabetically (note that hints_displayed contains indices into hints)
            self.hints_displayed
                .sort_by(|&a, &b| hints[a].cmp(&hints[b]));
        }

        if !self.hints_displayed.is_empty() {
            self.selected_hint = 0;
        }
    }

    fn get_hint_key_and_rest(&self) -> Option<(&str, &str)> {
        let cur_line = self.cur_line.trim();
        if cur_line.is_empty() {
            return None;
        }
        let mut split = cur_line.split_whitespace();
        let cur_cmd = split.next().unwrap().trim();
        let rest = split.next();
        if let Some(rest) = rest {
            Some((cur_cmd, rest.trim()))
        } else {
            Some(("", cur_cmd))
        }
    }

    fn del_prev_char(&mut self) -> Option<char> {
        debug_assert!(self.cur_pos <= self.cur_line.len());

        if self.cur_pos > 0 {
            self.cur_pos -= 1;
        } else {
            return None;
        }

        if self.cur_pos < self.cur_line.len() {
            Some(self.cur_line.remove(self.cur_pos))
        } else {
            None
        }
    }

    fn del_prev_word(&mut self) {
        if self.cur_pos == 0 {
            return;
        }
        let mut prev_was_ws = self.cur_line.chars().nth(self.cur_pos - 1) == Some(' ');
        while self.cur_pos > 0 {
            match self.del_prev_char() {
                Some('/') => break,
                Some(' ') => {
                    if !prev_was_ws {
                        self.cur_line.push(' ');
                        self.cur_pos += 1;
                        break;
                    } else {
                        prev_was_ws = self.cur_pos > 0
                            && self.cur_line.chars().nth(self.cur_pos - 1) == Some(' ');
                    }
                }
                _ => {}
            }
        }
    }

    fn del_next_char(&mut self) -> Option<char> {
        debug_assert!(self.cur_pos <= self.cur_line.len());

        if self.cur_pos <= self.cur_line.len() {
            Some(self.cur_line.remove(self.cur_pos))
        } else {
            None
        }
    }

    fn del_next_word(&mut self) {
        // @Improve: this works pretty bad, should be more like move_one_word().
        while self.cur_pos < self.cur_line.len() && self.del_next_char() != Some(' ') {}
    }

    fn move_one_char(&mut self, sign: i8) {
        let sign = sign.signum();
        if sign < 0 {
            if self.cur_pos > 0 {
                self.cur_pos -= 1;
            }
        } else if self.cur_pos < self.cur_line.len() {
            self.cur_pos += 1;
        }
    }

    fn move_one_word(&mut self, sign: i8) {
        let sign = sign.signum();

        fn char_at(s: &str, idx: usize) -> char {
            s.chars().nth(idx).unwrap()
        }

        let mut crossed_ws = false;
        if sign < 0 {
            while self.cur_pos > 0 {
                self.cur_pos -= 1;
                if crossed_ws {
                    if !char_at(&self.cur_line, self.cur_pos).is_whitespace() {
                        self.cur_pos += 1;
                        break;
                    }
                } else if char_at(&self.cur_line, self.cur_pos).is_whitespace() {
                    crossed_ws = true;
                }
            }
        } else {
            while self.cur_pos < self.cur_line.len() {
                self.cur_pos += 1;
                if self.cur_pos == self.cur_line.len() {
                    break;
                }
                if crossed_ws {
                    if !char_at(&self.cur_line, self.cur_pos).is_whitespace() {
                        self.cur_pos -= 1;
                        break;
                    }
                } else if char_at(&self.cur_line, self.cur_pos).is_whitespace() {
                    crossed_ws = true;
                }
            }
        }
    }

    pub fn draw(
        &self,
        window: &mut Render_Window_Handle,
        gres: &mut gfx::Gfx_Resources,
        config: &Config,
    ) {
        if self.status == Console_Status::Closed {
            return;
        }

        let ui_scale = self.cfg.ui_scale.read(config);
        let font_size =
            u16::try_from((self.cfg.font_size.read(config) as f32 * ui_scale) as u32).unwrap();

        let pad_x = self.cfg.pad_x.read(config) * ui_scale;
        let linesep = self.cfg.linesep.read(config) * ui_scale;
        let linesep = linesep + font_size as f32;

        let opacity = (self.cfg.opacity.read(config) * 255.0) as u8;
        let cur_line_opacity = (self.cfg.cur_line_opacity.read(config) * 255.0) as u8;

        // Draw background
        let Vec2u { x, y } = self.pos;
        let Vec2u { x: w, y: h } = self.size;
        render::render_rect(
            window,
            Rect::new(x, y, w, h - linesep as u32),
            colors::rgba(0, 0, 0, opacity),
        );
        render::render_rect(
            window,
            Rect::new(x, h - linesep as u32, w, linesep as u32),
            colors::rgba(30, 30, 30, cur_line_opacity),
        );

        // Draw cur line
        let font = gres.get_font(self.cfg.font);
        let text = render::create_text(window, &self.cur_line, font, font_size);
        let mut pos = v2!(x as f32 + pad_x, (y + h) as f32 - linesep);
        let Vec2f { x: line_w, .. } = render::get_text_size(&text);
        render::render_text(window, &text, colors::WHITE, pos);

        // Draw cursor
        let cursor = Rect::new(
            pad_x + (self.cur_pos as f32 / self.cur_line.len().max(1) as f32) * line_w,
            pos.y + linesep,
            font_size as f32 * 0.6,
            font_size as f32 * 0.1,
        );
        render::render_rect(window, cursor, colors::WHITE);

        // Draw output
        {
            let mut pos = pos - Vec2f::new(0.0, linesep);
            for (line, color) in self.output.iter().rev() {
                let text = render::create_text(window, line, font, font_size);
                render::render_text(window, &text, *color, pos);
                pos.y -= linesep;
                if pos.y < -linesep {
                    break;
                }
            }
        }

        // Draw hints
        let mut texts = Vec::with_capacity(self.hints_displayed.len());
        if let Some((cmd, _)) = self.get_hint_key_and_rest() {
            if let Some(hints) = &self.hints.get(cmd) {
                for (i, idx) in self.hints_displayed.iter().enumerate() {
                    let text = render::create_text(
                        window,
                        &hints[*idx],
                        font,
                        (font_size as f32 * 0.9) as _,
                    );
                    let color = if i == self.selected_hint {
                        colors::YELLOW
                    } else {
                        colors::rgba(200, 200, 200, 255)
                    };
                    texts.push((text, color));
                }
            }
        }

        // Draw hints background
        {
            let position = pos - Vec2f::new(0.0, linesep * texts.len() as f32);
            let tot_height = linesep * texts.len() as f32;
            render::render_rect(
                window,
                Rect::new(position.x, position.y, w as f32, tot_height),
                colors::rgb(20, 20, 20),
            );
        }

        for (text, color) in texts {
            pos.y -= linesep;
            render::render_text(window, &text, color, pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn console_del_prev_word() {
        let mut console = Console::new();
        console.init(Console_Config::default());

        console.cur_line = String::from("foooo");
        console.cur_pos = console.cur_line.len();
        console.del_prev_word();
        assert_eq!(console.cur_line.as_str(), "");
        assert_eq!(console.cur_pos, console.cur_line.len());

        console.del_prev_word();
        assert_eq!(console.cur_line.as_str(), "");
        assert_eq!(console.cur_pos, console.cur_line.len());

        console.cur_line = String::from("foo bar bazzz");
        console.cur_pos = console.cur_line.len();
        console.del_prev_word();
        assert_eq!(console.cur_line.as_str(), "foo bar ");
        assert_eq!(console.cur_pos, console.cur_line.len());

        console.del_prev_word();
        assert_eq!(console.cur_line.as_str(), "foo ");
        assert_eq!(console.cur_pos, console.cur_line.len());

        console.cur_line = String::from("foo      ");
        console.cur_pos = console.cur_line.len();
        console.del_prev_word();
        assert_eq!(console.cur_line.as_str(), "");
        assert_eq!(console.cur_pos, console.cur_line.len());

        console.cur_line = String::from(" ");
        console.cur_pos = console.cur_line.len();
        console.del_prev_word();
        assert_eq!(console.cur_line.as_str(), "");
        assert_eq!(console.cur_pos, console.cur_line.len());

        console.cur_line = String::from("   foo  bar");
        console.cur_pos = console.cur_line.len();
        console.del_prev_word();
        assert_eq!(console.cur_line.as_str(), "   foo  ");
        assert_eq!(console.cur_pos, console.cur_line.len());

        console.del_prev_word();
        assert_eq!(console.cur_line.as_str(), "   ");
        assert_eq!(console.cur_pos, console.cur_line.len());

        console.del_prev_word();
        assert_eq!(console.cur_line.as_str(), "");
        assert_eq!(console.cur_pos, console.cur_line.len());
    }
}
