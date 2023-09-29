use super::element::{Debug_Element, Draw_Args, Update_Args, Update_Res};
use inle_cfg::Cfg_Var;
use inle_common::colors;
use inle_diagnostics::log::Logger;
use inle_gfx::render;
use inle_gfx::res::Font_Handle;
use inle_input::input_state::Action_Kind;
use inle_input::mouse;
use inle_math::rect::{Rect, Rectf};
use inle_math::vector::{Vec2f, Vec2u};
use std::borrow::{Borrow, Cow};
use std::cell::Cell;
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

#[derive(Default)]
struct Debug_Line {
    pub file: &'static str,
    pub line: u32,
    pub tag: &'static str,
    pub msg: String,
    pub required_lines: Cell<u16>, // This is computed lazily once we render the text for the first time
}

struct Log_Window_Logger {
    msg_sender: Sender<Debug_Line>,
}

impl Log_Window_Logger {
    pub fn new(msg_sender: Sender<Debug_Line>) -> Self {
        Self { msg_sender }
    }
}

#[derive(Default)]
pub struct Log_Window {
    // This pos starts from the header
    pub pos: Vec2u,
    // NOTE: size does not include the header size
    pub size: Vec2u,

    lines: VecDeque<Debug_Line>,
    first_line: usize,
    // We're saving this so we can easily scroll to the end the frame after we receive new messages.
    // Since we don't know how many lines will every msg take, we don't scroll right after we receive them
    // but the frame after, if we're at the end.
    is_at_end: bool,
    max_lines: Cell<Option<u16>>, // This is computed lazily once we fill the window for the first time
    mouse_pos: Vec2f,

    cfg: Log_Window_Config,
    msg_receiver: Option<Receiver<Debug_Line>>,
}

#[derive(Clone, Default)]
pub struct Log_Window_Config {
    pub font: Font_Handle,
    pub max_lines: usize,

    pub ui_scale: Cfg_Var<f32>,
    pub font_size: Cfg_Var<u32>,
    pub pad_x: Cfg_Var<f32>,
    pub pad_y: Cfg_Var<f32>,
    pub linesep: Cfg_Var<f32>,
    pub scrolled_lines: Cfg_Var<u32>, // How much we scroll with wheel
    pub page_scrolled_lines: Cfg_Var<u32>, // How much we scroll with page_up/down
    pub header_height: Cfg_Var<u32>,
    pub title_font_size: Cfg_Var<u32>,
    pub title: Cow<'static, str>,
}

impl Log_Window {
    pub fn new(cfg: &Log_Window_Config) -> Self {
        Self {
            cfg: cfg.clone(),
            is_at_end: true,
            ..Default::default()
        }
    }

    pub fn create_logger(&mut self) -> Box<dyn Logger> {
        let (send, recv) = channel();
        self.msg_receiver = Some(recv);
        Box::new(Log_Window_Logger::new(send))
    }

    fn compute_starting_line_and_subline(&self) -> (usize, u16) {
        let first = self.first_line as u16;
        let mut n_real_lines_skipped = 0;
        for (i, line) in self.lines.iter().enumerate() {
            n_real_lines_skipped += line.required_lines.get();
            if n_real_lines_skipped >= first {
                return (
                    i,
                    (line.required_lines.get()) - (n_real_lines_skipped - first),
                );
            }
        }
        (0, 0)
    }

    fn scroll_up(&mut self, n_lines: usize) {
        self.first_line = self.first_line.saturating_sub(n_lines);
    }

    fn scroll_down(&mut self, n_lines: usize) {
        if let Some(max_lines) = self.max_lines.get() {
            let total_lines_stored: usize = self
                .lines
                .iter()
                .map(|line| line.required_lines.get() as usize)
                .sum();
            self.first_line = (self.first_line + n_lines)
                .min(total_lines_stored.saturating_sub(max_lines as usize - 1));
        }
    }
}

fn get_tag_color(tag: &str) -> colors::Color {
    match tag {
        "ERROR" => colors::RED,
        "WARNING" => colors::YELLOW,
        "DEBUG" => colors::GRAY,
        _ => colors::WHITE,
    }
}

const ELLIPSIS: &str = "-";

// @Temporary: this function should be common
fn create_wrapped_text(
    window: &mut inle_gfx::render_window::Render_Window_Handle,
    txt: &str,
    font: &render::Font,
    font_size: u16,
    line_width: f32,
) -> Vec<render::Text> {
    trace!("create_wrapped_text");

    // @Speed: this algorithm could probably be improved
    let mut texts = Vec::default();

    let ellipsis_text = render::create_text(window, ELLIPSIS, font, font_size);
    let ellipsis_text_width = render::get_text_size(&ellipsis_text).x;

    let text = render::create_text(window, txt, font, font_size);
    let mut candidate_texts = vec![text];

    let true_line_width = line_width - ellipsis_text_width;
    while let Some(text) = candidate_texts.pop() {
        let text_width = render::get_text_size(&text).x;
        if text_width < true_line_width {
            texts.push(text);
            continue;
        }

        let string = render::get_text_string(&text);
        let estimate_wrap_idx =
            (true_line_width / text_width * string.chars().count() as f32) as usize;
        let estimate_wrap_idx = estimate_wrap_idx.saturating_sub(1);
        let (s1, s2) = string.split_at(estimate_wrap_idx);

        let t1 = render::create_text(window, &format!("{}{}", s1, ELLIPSIS), font, font_size);
        let t2 = render::create_text(window, s2, font, font_size);

        candidate_texts.push(t2);
        candidate_texts.push(t1);
    }

    texts
}

// Used to scroll down to the end. Represents the number of lines we want to scroll down.
const A_LOT: usize = 999999;

impl Debug_Element for Log_Window {
    fn update(
        &mut self,
        Update_Args {
            window,
            input_state,
            config,
            ..
        }: Update_Args,
    ) -> Update_Res {
        trace!("log_window::update");

        self.mouse_pos = Vec2f::from(mouse::mouse_pos_in_window(
            window,
            &input_state.raw.mouse_state,
        ));

        // @Incomplete: allow dragging
        // @Incomplete: allow resizing

        if self.is_at_end {
            self.scroll_down(A_LOT);
        }

        let actions = &input_state.processed.game_actions;

        let scrolled_lines = self.cfg.scrolled_lines.read(config) as usize;
        let page_scrolled_lines = self.cfg.page_scrolled_lines.read(config) as usize;

        if actions.contains(&(sid!("scroll_up"), Action_Kind::Pressed)) {
            self.scroll_up(scrolled_lines);
        } else if actions.contains(&(sid!("scroll_down"), Action_Kind::Pressed)) {
            self.scroll_down(scrolled_lines);
        } else if actions.contains(&(sid!("page_up"), Action_Kind::Pressed)) {
            self.scroll_up(page_scrolled_lines);
        } else if actions.contains(&(sid!("page_down"), Action_Kind::Pressed)) {
            self.scroll_down(page_scrolled_lines);
        } else if actions.contains(&(sid!("page_home"), Action_Kind::Pressed)) {
            self.first_line = 0;
        } else if actions.contains(&(sid!("page_end"), Action_Kind::Pressed)) {
            self.scroll_down(A_LOT);
        }

        self.is_at_end = if let Some(max_lines) = self.max_lines.get() {
            let total_lines_stored: usize = self
                .lines
                .iter()
                .map(|line| line.required_lines.get() as usize)
                .sum();
            self.first_line == total_lines_stored.saturating_sub(max_lines as usize)
        } else {
            true
        };

        if self.msg_receiver.is_none() {
            return Update_Res::Stay_Enabled;
        }

        let recv = self.msg_receiver.as_mut().unwrap();
        let mut should_disconnect = false;
        loop {
            match recv.try_recv() {
                Ok(msg) => {
                    self.lines.push_back(msg);
                    if self.lines.len() > self.cfg.max_lines {
                        self.lines.pop_front();
                    }
                    debug_assert!(self.lines.len() <= self.cfg.max_lines);
                }
                Err(TryRecvError::Disconnected) => {
                    should_disconnect = true;
                    break;
                }
                Err(_) => break,
            }
        }

        if should_disconnect {
            lwarn!("Log_Window disconnected from logging system.");
            self.msg_receiver = None;
        }

        Update_Res::Stay_Enabled
    }

    fn draw(
        &self,
        Draw_Args {
            window,
            gres,
            config,
            ..
        }: Draw_Args,
    ) {
        trace!("log_window::draw");

        let Vec2u { x, y } = self.pos;
        let Vec2u { x: w, y: h } = self.size;

        let font = gres.get_font(self.cfg.font);
        let ui_scale = self.cfg.ui_scale.read(config);
        let pad_x = self.cfg.pad_x.read(config) * ui_scale;
        let pad_y = self.cfg.pad_y.read(config) * ui_scale;
        let font_size =
            u16::try_from((self.cfg.font_size.read(config) as f32 * ui_scale) as u32).unwrap();
        let title_font_size =
            u16::try_from((self.cfg.title_font_size.read(config) as f32 * ui_scale) as u32)
                .unwrap();
        let linesep = self.cfg.linesep.read(config) * ui_scale;
        let header_height = (self.cfg.header_height.read(config) as f32 * ui_scale) as u32;

        // Render header
        render::render_rect(
            window,
            Rect::new(x, y, w, header_height),
            colors::rgb(40, 40, 40),
        );
        {
            let text = render::create_text(window, self.cfg.title.borrow(), font, title_font_size);
            let text_height = render::get_text_size(&text).y;
            render::render_text(
                window,
                &text,
                colors::WHITE,
                Vec2f::from(self.pos) + v2!(pad_x, 0.5 * (header_height as f32 - text_height)),
            );
        }

        // Render main content background
        render::render_rect(
            window,
            Rect::new(x, y + header_height, w, h),
            colors::rgb(20, 20, 20),
        );

        let base_pos = Vec2f::from(self.pos) + v2!(pad_x, pad_y + header_height as f32);

        // Compute starting line
        let (first_line, first_subline) = self.compute_starting_line_and_subline();

        // Render main content
        let mut y = 0.;
        let mut tot_lines_drawn = 0u16;
        'outer: for (i, line) in self.lines.iter().skip(first_line).enumerate() {
            if let Some(max_lines) = self.max_lines.get() {
                if i == max_lines as usize {
                    break;
                }
            }

            // @Speed: we're recomputing the wrapping everytime just to keep the code a bit simpler.
            let texts = create_wrapped_text(
                window,
                &line.msg,
                font,
                font_size,
                self.size.x as f32 - pad_x,
            );
            debug_assert!(texts.len() < u16::MAX as usize);
            line.required_lines.set(texts.len() as u16);

            let mut color = get_tag_color(line.tag);

            let line_height = render::get_text_size(&texts[0]).y;
            let min_pos = base_pos + v2!(0., y);
            let max_pos = base_pos
                + v2!(
                    self.size.x as f32,
                    y + texts.len() as f32 * (line_height + linesep)
                );
            let is_hovered =
                Rectf::from_topleft_botright(min_pos, max_pos).contains(self.mouse_pos);

            if is_hovered {
                color = colors::darken(color, -0.5);
            }

            // Draw every text in the wrapped line
            let texts_to_skip = if i == 0 { first_subline } else { 0 };
            for text in texts.into_iter().skip(texts_to_skip.into()) {
                if y + line_height > self.size.y as f32 - pad_y {
                    debug_assert!(i < u16::MAX as usize);
                    if self.max_lines.get().is_none() {
                        self.max_lines.set(Some(tot_lines_drawn));
                    }
                    break 'outer;
                }

                let pos = base_pos + v2!(0., y);
                render::render_text(window, &text, color, pos);

                y += line_height + linesep;
                tot_lines_drawn += 1;
            }
        }

        // @Incomplete: render scrollbar
    }
}

impl Logger for Log_Window_Logger {
    fn log(&mut self, file: &'static str, line: u32, tag: &'static str, msg: &str) {
        if tag == "VERBOSE" {
            return;
        }
        // Do not unwrap since this fails when closing the game
        let _ = self.msg_sender.send(Debug_Line {
            file,
            line,
            tag,
            msg: String::from(msg),
            required_lines: Cell::new(1),
        });
    }

    fn set_log_file_line(&mut self, _l: bool) {}
}
