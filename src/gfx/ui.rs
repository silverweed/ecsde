use crate::core;
use crate::core::common::colors::{self, Color};
use crate::core::common::rect::Rect;
use crate::core::common::vector::{to_framework_vec, Vec2f};
use crate::core::common::Maybe_Error;
use crate::core::env::Env_Info;
use crate::gfx;
use crate::gfx::window::Window_Handle;
use crate::resources;
use crate::resources::gfx::{Font_Handle, Gfx_Resources, Texture_Handle};
use sfml::graphics::Text;
use sfml::graphics::Transformable;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::vec::Vec;

pub enum UI_Request {
    Add_Fadeout_Text(String),
}

struct Fadeout_Text {
    pub text: String,
    pub time: Duration,
}

pub struct UI_System {
    fadeout_text_font: Font_Handle,
    fadeout_texts: Vec<Fadeout_Text>,
    ui_requests_rx: Receiver<UI_Request>,
    fadeout_time: Duration,
}

impl UI_System {
    const FADEOUT_TEXT_PAD_X: i32 = 5;
    const FADEOUT_TEXT_PAD_Y: i32 = 5;
    const FADEOUT_TEXT_ROW_HEIGHT: usize = 20;
    const FADEOUT_TEXT_FONT_SIZE: u16 = 16;
    const FADEOUT_TEXT_FONT: &'static str = "Hack-Regular.ttf";
    const DEFAULT_FADEOUT_TIME_MS: u64 = 3000;

    pub fn new(req_rx: Receiver<UI_Request>) -> UI_System {
        UI_System {
            fadeout_text_font: None,
            fadeout_texts: vec![],
            ui_requests_rx: req_rx,
            fadeout_time: Duration::from_millis(Self::DEFAULT_FADEOUT_TIME_MS),
        }
    }

    pub fn init(&mut self, env: &Env_Info, gres: &mut Gfx_Resources) -> Maybe_Error {
        self.fadeout_text_font =
            gres.load_font(&resources::gfx::font_path(env, Self::FADEOUT_TEXT_FONT));
        if self.fadeout_text_font.is_none() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to load font for UI!",
            )));
        }

        Ok(())
    }

    pub fn update(&mut self, dt: &Duration, window: &mut Window_Handle, gres: &mut Gfx_Resources) {
        self.handle_ui_requests();
        self.update_fadeout_texts(dt);
        self.draw_fadeout_texts(window, gres);
    }

    fn handle_ui_requests(&mut self) {
        let iter = self.ui_requests_rx.try_iter().collect::<Vec<UI_Request>>();
        for req in iter {
            match req {
                UI_Request::Add_Fadeout_Text(txt) => self.add_fadeout_text(txt),
                _ => unreachable!(),
            }
        }
    }

    fn update_fadeout_texts(&mut self, dt: &Duration) {
        let fadeout_time = &self.fadeout_time;
        let n_expired = self
            .fadeout_texts
            .iter_mut()
            .map(|t| {
                t.time += *dt;
                &t.time
            })
            .filter(|&time| time >= fadeout_time)
            .count();
        self.fadeout_texts.drain(0..n_expired);
    }

    fn draw_fadeout_texts(&mut self, window: &mut Window_Handle, gres: &mut Gfx_Resources) {
        let fadeout_time = self.fadeout_time;

        for (i, fadeout_text) in self.fadeout_texts.iter().enumerate() {
            let d = core::time::duration_ratio(&fadeout_text.time, &fadeout_time);
            let alpha = 255 - (d * d * 255.0f32) as u8;
            let text = {
                let mut text = Text::new(
                    &fadeout_text.text,
                    gres.get_font(self.fadeout_text_font),
                    Self::FADEOUT_TEXT_FONT_SIZE.into(),
                );
                text.set_fill_color(&colors::rgba(255, 255, 255, alpha));
                text.set_position(to_framework_vec(Vec2f::new(
                    Self::FADEOUT_TEXT_PAD_X as f32,
                    Self::FADEOUT_TEXT_PAD_Y as f32 + (i * Self::FADEOUT_TEXT_ROW_HEIGHT) as f32,
                )));
                text
            };

            gfx::render::render_text(window, &text);
        }
    }

    pub fn add_fadeout_text(&mut self, txt: String) {
        self.fadeout_texts.push(Fadeout_Text {
            text: txt,
            time: Duration::new(0, 0),
        });
    }
}
