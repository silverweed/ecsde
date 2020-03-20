use crate::common::colors;
use crate::common::rect::Rect;
use crate::common::vector::{Vec2f, Vec2u};
use crate::core::env::Env_Info;
use crate::gfx::render;
use crate::gfx::window::Window_Handle;
use crate::input::bindings::keyboard;
use crate::input::input_system::Input_Raw_Event;
use crate::resources::gfx;
use std::collections::HashMap;
use std::fs::File;
use std::io;

mod history;

use history::{Direction, History};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Console_Status {
    Open,
    Closed,
}

const OUTPUT_SIZE: usize = 250;
const HIST_SIZE: usize = 50;
const COLOR_HISTORY: colors::Color = colors::rgba(200, 200, 200, 200);
const HIST_FILE: &str = ".console_hist.txt";

pub struct Console {
    pub status: Console_Status,
    pub pos: Vec2u,
    pub size: Vec2u,
    pub font_size: u16,
    // This should be (externally) set equal to the "toggle_console" action keys.
    pub toggle_console_keys: Vec<keyboard::Key>,

    font: gfx::Font_Handle,

    cur_line: String,
    cur_pos: usize,

    enqueued_cmd: Option<String>,

    output: Vec<(String, colors::Color)>,
    history: History<String>,

    // { cur_cmd => hints relative to it } (empty string => all commands)
    // @Speed: we should avoid having more copies of the same hint set if more commands share it.
    hints: HashMap<String, Vec<String>>,
    hints_displayed: Vec<usize>,
    selected_hint: usize,
}

pub fn save_console_hist(console: &Console) -> io::Result<()> {
    use std::io::{prelude::*, BufWriter};

    let mut file = BufWriter::new(File::create(HIST_FILE)?);
    for line in &console.history {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}

pub fn load_console_hist(console: &mut Console) -> io::Result<()> {
    use std::io::{prelude::*, BufReader};

    let mut file = BufReader::new(File::open(HIST_FILE)?);
    for i in 0..HIST_SIZE {
        let mut line = String::new();
        match file.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => console.history.push(line),
            Err(err) => {
                return Err(err);
            }
        }
    }

    Ok(())
}

impl Console {
    pub fn new() -> Self {
        Self {
            status: Console_Status::Closed,
            pos: Vec2u::default(),
            size: Vec2u::default(),
            font_size: 14,
            cur_line: String::new(),
            cur_pos: 0,
            font: None,
            toggle_console_keys: vec![],
            enqueued_cmd: None,
            output: Vec::with_capacity(OUTPUT_SIZE),
            history: History::with_capacity(HIST_SIZE),
            hints: HashMap::new(),
            hints_displayed: vec![],
            selected_hint: 0,
        }
    }

    pub fn init(&mut self, gres: &mut gfx::Gfx_Resources, env: &Env_Info) {
        const FONT_NAME: &str = "Hack-Regular.ttf";
        self.font = gres.load_font(&gfx::font_path(env, FONT_NAME));
    }

    pub fn toggle(&mut self) -> Console_Status {
        use Console_Status::*;
        self.status = match self.status {
            Open => Closed,
            Closed => Open,
        };
        self.status
    }

    pub fn update(&mut self, events: &[Input_Raw_Event]) {
        for event in events {
            self.process_event(*event);
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
            .or_insert_with(|| vec![])
            .extend(hints);
    }

    pub fn output_line<T: ToString>(&mut self, line: T, color: colors::Color) {
        self.output.push((line.to_string(), color));
    }

    #[cfg(feature = "use-sfml")]
    fn process_event(&mut self, event: Input_Raw_Event) {
        use keyboard::Key;
        use sfml::window::Event;

        debug_assert!(self.cur_pos <= self.cur_line.len());

        let mut line_changed = false;

        match event {
            Event::KeyPressed { code, .. } if self.toggle_console_keys.contains(&code) => {
                self.status = Console_Status::Closed;
            }
            Event::KeyPressed {
                code: Key::BackSpace,
                ctrl,
                ..
            } => {
                if ctrl {
                    self.del_prev_word();
                } else {
                    self.del_prev_char();
                }
                line_changed = true;
            }
            Event::KeyPressed { code: Key::Up, .. } => {
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
            }
            Event::KeyPressed {
                code: Key::Down, ..
            } => {
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
            }
            Event::KeyPressed {
                code: Key::Left,
                ctrl,
                ..
            } => {
                if ctrl {
                    self.move_one_word(-1);
                } else {
                    self.move_one_char(-1);
                }
            }
            Event::KeyPressed {
                code: Key::Right,
                ctrl,
                ..
            } => {
                if ctrl {
                    self.move_one_word(1);
                } else {
                    self.move_one_char(1);
                }
            }
            Event::KeyPressed {
                code: Key::A,
                ctrl: true,
                ..
            } => {
                self.cur_pos = 0;
            }
            Event::KeyPressed {
                code: Key::Home, ..
            } => {
                self.cur_pos = 0;
            }
            Event::KeyPressed {
                code: Key::E,
                ctrl: true,
                ..
            } => {
                self.cur_pos = self.cur_line.len();
            }
            Event::KeyPressed { code: Key::End, .. } => {
                self.cur_pos = self.cur_line.len();
            }
            Event::KeyPressed {
                code: Key::W,
                ctrl: true,
                ..
            } => {
                self.del_prev_word();
                line_changed = true;
            }
            Event::KeyPressed {
                code: Key::Delete,
                ctrl,
                ..
            } => {
                if ctrl {
                    self.del_next_word();
                } else {
                    self.del_next_char();
                }
                line_changed = true;
            }
            Event::KeyPressed {
                code: Key::K,
                ctrl: true,
                ..
            } => {
                self.cur_line.truncate(self.cur_pos);
                line_changed = true;
            }
            Event::KeyPressed {
                code: Key::D,
                ctrl: true,
                ..
            } => {
                self.cur_line.clear();
                self.cur_pos = 0;
                line_changed = true;
            }
            Event::KeyPressed {
                code: Key::Return, ..
            } => {
                self.commit_line();
                line_changed = true;
            }
            Event::KeyPressed { code: Key::Tab, .. } => {
                if !self.hints_displayed.is_empty() {
                    // @Improve: this is a pretty rudimentary behaviour: consider improving.
                    let (cmd, _) = self.get_hint_key_and_rest().unwrap();
                    self.cur_line = cmd.to_string()
                        + if cmd.is_empty() { "" } else { " " }
                        + &self.hints[cmd][self.hints_displayed[self.selected_hint]]
                        + " ";
                    self.cur_pos = self.cur_line.len();
                }
                line_changed = true;
            }
            Event::KeyPressed { code, shift, .. } => {
                if let Some(c) = keyboard::key_to_char(code, shift) {
                    self.cur_line.insert(self.cur_pos, c);
                    self.cur_pos += 1;
                }
                line_changed = true;
            }
            _ => {}
        }
        if line_changed {
            self.update_hints();
        }
    }

    fn commit_line(&mut self) {
        let cmdline = self.cur_line.trim().to_string();
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
        // @Improve: sort by relevance
        if let Some(hints) = self.hints.get(&cmd) {
            for (i, hint) in hints.iter().enumerate() {
                if hint.contains(&rest) {
                    self.hints_displayed.push(i);
                }
            }
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
        while self.cur_pos > 0 {
            match self.del_prev_char() {
                Some(' ') | Some('/') => break,
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

    pub fn draw(&self, window: &mut Window_Handle, gres: &mut gfx::Gfx_Resources) {
        if self.status == Console_Status::Closed {
            return;
        }

        // @Temporary
        let pad_x = 5.0;
        let linesep = self.font_size as f32 * 1.2;

        // Draw background
        let Vec2u { x, y } = self.pos;
        let Vec2u { x: w, y: h } = self.size;
        render::render_rect(
            window,
            Rect::new(x, y, w, h - linesep as u32),
            colors::rgba(0, 0, 0, 150),
        );
        render::render_rect(
            window,
            Rect::new(x, h - linesep as u32, w, linesep as u32),
            colors::rgba(30, 30, 30, 200),
        );

        // Draw cur line
        let font = gres.get_font(self.font);
        let mut text = render::create_text(&self.cur_line, font, self.font_size);
        let mut pos = Vec2f::from(self.pos) + Vec2f::new(pad_x, self.size.y as f32 - linesep);
        let Vec2f { x: line_w, .. } = render::get_text_size(&text);
        render::render_text(window, &mut text, colors::WHITE, pos);

        // Draw cursor
        let cursor = Rect::new(
            pad_x + (self.cur_pos as f32 / self.cur_line.len().max(1) as f32) * line_w as f32,
            pos.y + linesep,
            self.font_size as f32 * 0.6,
            self.font_size as f32 * 0.1,
        );
        render::render_rect(window, cursor, colors::WHITE);

        // Draw output
        {
            let mut pos = pos - Vec2f::new(0.0, linesep as f32);
            for (line, color) in self.output.iter().rev() {
                let mut text = render::create_text(line, font, self.font_size);
                render::render_text(window, &mut text, *color, pos);
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
                    let text =
                        render::create_text(&hints[*idx], font, (self.font_size as f32 * 0.9) as _);
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
            let position = pos - Vec2f::new(0.0, linesep as f32 * texts.len() as f32);
            let tot_height = linesep as f32 * texts.len() as f32;
            render::render_rect(
                window,
                Rect::new(position.x, position.y, w as f32, tot_height),
                colors::rgb(20, 20, 20),
            );
        }

        for (mut text, color) in texts {
            pos.y -= linesep;
            render::render_text(window, &mut text, color, pos);
        }
    }
}
