use super::element::{Debug_Element, Draw_Args, Update_Args};
use inle_common::colors;
use inle_diagnostics::log::Logger;
use inle_gfx::render;
use inle_math::rect::Rect;
use inle_math::vector::{Vec2f, Vec2u};
use inle_resources::gfx::Font_Handle;
use std::cell::Cell;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

#[derive(Default)]
struct Debug_Line {
    pub tag: &'static str,
    pub msg: String,
    pub required_lines: Cell<u8>, // This is computed lazily once we render the text for the first time
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
    pub pos: Vec2u,
    pub size: Vec2u,

    lines: Vec<Debug_Line>,
    first_line: usize,
    max_lines: Cell<Option<u16>>, // This is computed lazily once we fill the window for the first time
    is_at_end: Cell<bool>,

    cfg: Log_Window_Config,
    msg_receiver: Option<Receiver<Debug_Line>>,
}

#[derive(Default)]
pub struct Log_Window_Config {
    pub font: Font_Handle,
    pub font_size: u16,
    pub pad_x: f32,
    pub pad_y: f32,
    pub linesep: f32,
}

impl Log_Window {
    pub fn new(cfg: Log_Window_Config) -> Self {
        Self {
            cfg,
            ..Default::default()
        }
    }

    pub fn create_logger(&mut self) -> Box<dyn Logger> {
        let (send, recv) = channel();
        self.msg_receiver = Some(recv);
        Box::new(Log_Window_Logger::new(send))
    }
}

fn get_tag_color(tag: &str) -> colors::Color {
    match tag {
        "ERROR" => colors::RED,
        "WARNING" => colors::YELLOW,
        _ => colors::WHITE,
    }
}

const ELLIPSIS: &str = "-";

// @Temporary: this function should be common
fn create_wrapped_text<'a>(
    txt: &str,
    font: &'a render::Font<'a>,
    font_size: u16,
    line_width: f32,
) -> Vec<render::Text<'a>> {
    // @Speed: this algorithm could probably be improved
    let mut texts = Vec::default();

    let ellipsis_text = Some(render::create_text(ELLIPSIS, font, font_size));
    let ellipsis_text_width = render::get_text_size(ellipsis_text.as_ref().unwrap()).x;

    let text = render::create_text(txt, font, font_size);
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

        let t1 = render::create_text(&format!("{}{}", s1, ELLIPSIS), font, font_size);
        let t2 = render::create_text(s2, font, font_size);

        candidate_texts.push(t2);
        candidate_texts.push(t1);
    }

    texts
}

impl Debug_Element for Log_Window {
    fn update(&mut self, Update_Args { .. }: Update_Args) {
        if self.msg_receiver.is_none() {
            return;
        }

        let is_at_end = self.is_at_end.get();
        let recv = self.msg_receiver.as_mut().unwrap();
        let mut should_disconnect = false;
        loop {
            match recv.try_recv() {
                Ok(msg) => {
                    self.lines.push(msg);
                    if is_at_end {
                        self.first_line += 1;
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    should_disconnect = true;
                    break;
                }
                Err(_) => break,
            }
        }

        if should_disconnect {
            self.msg_receiver = None;
        }
    }

    fn draw(&self, Draw_Args { window, gres, .. }: Draw_Args) {
        let Vec2u { x, y } = self.pos;
        let Vec2u { x: w, y: h } = self.size;
        render::render_rect(window, Rect::new(x, y, w, h), colors::rgb(20, 20, 20));

        let font = gres.get_font(self.cfg.font);
        let base_pos = Vec2f::from(self.pos) + v2!(self.cfg.pad_x, self.cfg.pad_y);
        let mut y = 0.;

        // @FIXME: we should consider the required_lines when we skip!
        let mut lines_to_skip_in_array = 0;
        let mut total_lines_skipped = 0;
        for line in &self.lines {
            if total_lines_skipped >= self.first_line {
                break;
            }
            total_lines_skipped += line.required_lines.get() as usize;
            lines_to_skip_in_array += 1;
        }
        let lines_to_skip_internally = total_lines_skipped - self.first_line;

        'outer: 
        for (i, line) in self.lines.iter().skip(lines_to_skip_in_array).enumerate() {
            if let Some(max_lines) = self.max_lines.get() {
                if i == max_lines as usize {
                    break;
                }
            }

            let texts = create_wrapped_text(
                &line.msg,
                font,
                self.cfg.font_size,
                self.size.x as f32 - self.cfg.pad_x,
            );
            debug_assert!(texts.len() < u8::MAX as usize);
            line.required_lines.set(texts.len() as u8);

            let color = get_tag_color(line.tag);

            for mut text in texts {
                let line_height = render::get_text_size(&text).y;
                if y + line_height > self.size.y as f32 - self.cfg.pad_y {
                    debug_assert!(i < u16::MAX as usize);
                    if self.max_lines.get().is_none() {
                        self.max_lines.set(Some(i as u16));
                        self.is_at_end.set(true);
                    }
                    break 'outer;
                }

                let pos = base_pos + v2!(0., y);
                render::render_text(window, &mut text, color, pos);

                y += line_height + self.cfg.linesep;
            }
        }
    }
}

impl Logger for Log_Window_Logger {
    fn log(&mut self, tag: &'static str, msg: &str) {
        self.msg_sender
            .send(Debug_Line {
                tag,
                msg: String::from(msg),
                required_lines: Cell::new(1),
            })
            .expect("Failed to send logger msg to Log_Window");
    }
}
