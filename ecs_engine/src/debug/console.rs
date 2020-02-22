use crate::cfg::Config;
use crate::common::colors;
use crate::common::rect::Rect;
use crate::common::vector::{Vec2f, Vec2u};
use crate::core::env::Env_Info;
use crate::gfx::render;
use crate::gfx::window::Window_Handle;
use crate::input::bindings::keyboard;
use crate::input::input_system::Input_Raw_Event;
use crate::input::provider::{Input_Provider, Input_Provider_Input};
use crate::resources::gfx;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Console_Status {
    Open,
    Closed,
}

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
    // @Temporary
    hints: Vec<String>,
    // @Temporary
    hints_displayed: Vec<usize>,
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
            hints: vec!["foo.bar".to_string(), "fooz.quuz".to_string()],
            hints_displayed: vec![],
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

    pub fn update(
        &mut self,
        window: &mut Input_Provider_Input,
        provider: &mut dyn Input_Provider,
        cfg: &Config,
    ) {
        provider.update(window, None, cfg);
        let events = provider.get_events();
        for event in events {
            self.process_event(*event);
        }
    }

    #[cfg(feature = "use-sfml")]
    fn process_event(&mut self, event: Input_Raw_Event) {
        use keyboard::Key;
        use sfml::window::Event;

        debug_assert!(self.cur_pos >= 0);
        debug_assert!(self.cur_pos <= self.cur_line.len());

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
            }
            Event::KeyPressed {
                code: Key::Up,
                ctrl,
                ..
            } => {
                // TODO: move in history
            }
            Event::KeyPressed {
                code: Key::Down,
                ctrl,
                ..
            } => {
                // TODO: move in history
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
                code: Key::E,
                ctrl: true,
                ..
            } => {
                self.cur_pos = self.cur_line.len();
            }
            Event::KeyPressed {
                code: Key::W,
                ctrl: true,
                ..
            } => {
                self.del_prev_word();
            }
            Event::KeyPressed {
                code: Key::K,
                ctrl: true,
                ..
            } => {
                self.cur_line.truncate(self.cur_pos);
            }
            Event::KeyPressed {
                code: Key::Return, ..
            } => {
                // TODO: commit line
            }
            Event::KeyPressed { code: Key::Tab, .. } => {
                if !self.hints_displayed.is_empty() {
                    // @Incomplete: this is a pretty rudimentary behaviour: consider improving.
                    self.cur_line = self.hints[self.hints_displayed[0]].clone();
                    self.cur_pos = self.cur_line.len();
                }
            }
            Event::KeyPressed { code, shift, .. } => {
                if let Some(c) = keyboard::key_to_char(code, shift) {
                    self.cur_line.insert(self.cur_pos, c);
                    self.cur_pos += 1;
                }
            }
            _ => {}
        }
        self.update_hints();
    }

    fn update_hints(&mut self) {
        self.hints_displayed.clear();

        if self.cur_line.is_empty() {
            return;
        }

        // @Incomplete: sort by relevance
        for (i, hint) in self.hints.iter().enumerate() {
            if hint.contains(&self.cur_line) {
                self.hints_displayed.push(i);
            }
        }
    }

    fn del_prev_char(&mut self) -> Option<char> {
        if self.cur_pos > 0 {
            self.cur_pos -= 1;
        }
        self.cur_line.pop()
    }

    fn del_prev_word(&mut self) {
        while self.cur_pos > 0 && self.del_prev_char() != Some(' ') {}
    }

    fn move_one_char(&mut self, sign: i8) {
        let sign = sign.signum();
        if sign < 0 {
            if self.cur_pos > 0 {
                self.cur_pos -= 1;
            }
        } else {
            if self.cur_pos < self.cur_line.len() {
                self.cur_pos += 1;
            }
        }
    }

    fn move_one_word(&mut self, sign: i8) {
        let sign = sign.signum();
        let mut crossed_ws = false;
        if sign < 0 {
            while self.cur_pos > 0 {
                self.cur_pos -= 1;
                if crossed_ws {
                    if !self
                        .cur_line
                        .chars()
                        .skip(self.cur_pos)
                        .next()
                        .unwrap()
                        .is_whitespace()
                    {
                        self.cur_pos += 1;
                        break;
                    }
                } else {
                    if self
                        .cur_line
                        .chars()
                        .skip(self.cur_pos)
                        .next()
                        .unwrap()
                        .is_whitespace()
                    {
                        crossed_ws = true;
                    }
                }
            }
        } else {
            while self.cur_pos < self.cur_line.len() {
                self.cur_pos += 1;
                if self.cur_pos == self.cur_line.len() {
                    break;
                }
                if crossed_ws {
                    if !self
                        .cur_line
                        .chars()
                        .skip(self.cur_pos)
                        .next()
                        .unwrap()
                        .is_whitespace()
                    {
                        self.cur_pos -= 1;
                        break;
                    }
                } else {
                    if self
                        .cur_line
                        .chars()
                        .skip(self.cur_pos)
                        .next()
                        .unwrap()
                        .is_whitespace()
                    {
                        crossed_ws = true;
                    }
                }
            }
        }
    }

    pub fn draw(&self, window: &mut Window_Handle, gres: &mut gfx::Gfx_Resources) {
        if self.status == Console_Status::Closed {
            return;
        }

        // Draw background
        let Vec2u { x, y } = self.pos;
        let Vec2u { x: w, y: h } = self.size;
        render::fill_color_rect(window, colors::rgba(0, 0, 0, 150), Rect::new(x, y, w, h));

        // @Temporary
        let pad_x = 5.0;
        let linesep = self.font_size as f32 * 1.2;

        // Draw cur line
        let font = gres.get_font(self.font);
        let mut text = render::create_text(&self.cur_line, font, self.font_size);
        let mut pos = Vec2f::from(self.pos) + Vec2f::new(pad_x, self.size.y as f32);
        render::render_text(window, &mut text, pos);

        // Draw cursor
        let Rect {
            width: line_w,
            height: line_h,
            ..
        } = render::get_text_local_bounds(&text);
        let cursor = Rect::new(
            pad_x + (self.cur_pos as f32 / self.cur_line.len().max(1) as f32) * line_w as f32,
            pos.y + linesep,
            self.font_size as f32 * 0.6,
            self.font_size as f32 * 0.1,
        );
        render::fill_color_rect(window, colors::WHITE, cursor);

        for idx in &self.hints_displayed {
            pos.y -= linesep;
            let mut text =
                render::create_text(&self.hints[*idx], font, (self.font_size as f32 * 0.9) as _);
            render::set_text_paint_props(&mut text, colors::rgba(200, 200, 200, 255));
            render::render_text(window, &mut text, pos);
        }
    }
}
