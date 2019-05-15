use crate::core;
use crate::core::common::Maybe_Error;
use crate::core::env::Env_Info;
use crate::resources::{self, Font_Handle, Resources, Texture_Handle};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{TextureQuery, WindowCanvas};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::vec::Vec;

pub enum UI_Request {
    Add_Fadeout_Text(String),
}

struct Fadeout_Text {
    pub texture: Texture_Handle,
    pub time: Duration,
}

type Fadeout_Text_Requests = Arc<Mutex<Vec<String>>>;

fn spawn_req_rx_listen_thread(
    req_rx: Receiver<UI_Request>,
    fadeout_text_requests: Fadeout_Text_Requests,
) -> JoinHandle<()> {
    thread::spawn(move || {
        req_rx_listen(req_rx, fadeout_text_requests);
    })
}

/// Worker thread that listens to incoming UI_Requests and appends them to the shared array
/// `fadeout_text_requests`. This array is used by the UI in its `update()` method to accomplish
/// the requests.
fn req_rx_listen(req_rx: Receiver<UI_Request>, fadeout_text_requests: Fadeout_Text_Requests) {
    while let Ok(req) = req_rx.recv() {
        match req {
            UI_Request::Add_Fadeout_Text(txt) => fadeout_text_requests.lock().unwrap().push(txt),
        }
    }
}

pub struct UI_System {
    fadeout_text_font: Font_Handle,
    fadeout_texts: Vec<Fadeout_Text>,
    fadeout_text_requests: Fadeout_Text_Requests,
    fadeout_time: Duration,
    req_tx: Sender<UI_Request>,
}

impl UI_System {
    const FADEOUT_TEXT_PAD_X: i32 = 5;
    const FADEOUT_TEXT_PAD_Y: i32 = 5;
    const FADEOUT_TEXT_FONT_SIZE: u16 = 16;
    const FADEOUT_TEXT_FONT: &'static str = "Hack-Regular.ttf";
    const DEFAULT_FADEOUT_TIME_MS: u64 = 3000;

    pub fn new() -> UI_System {
        let (req_tx, req_rx) = channel();
        let fadeout_text_requests = Arc::new(Mutex::new(vec![]));

        spawn_req_rx_listen_thread(req_rx, fadeout_text_requests.clone());

        UI_System {
            fadeout_text_font: None,
            fadeout_texts: vec![],
            fadeout_text_requests,
            fadeout_time: Duration::from_millis(Self::DEFAULT_FADEOUT_TIME_MS),
            req_tx,
        }
    }

    pub fn init(&mut self, env: &Env_Info, rsrc: &mut Resources) -> Maybe_Error {
        self.fadeout_text_font = rsrc.load_font(
            &resources::font_path(env, Self::FADEOUT_TEXT_FONT),
            Self::FADEOUT_TEXT_FONT_SIZE,
        );
        if self.fadeout_text_font.is_none() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to load font for UI!",
            )));
        }

        Ok(())
    }

    pub fn update(&mut self, dt: &Duration, canvas: &mut WindowCanvas, rsrc: &mut Resources) {
        self.handle_ui_requests(rsrc);
        self.update_fadeout_texts(dt);
        self.draw_fadeout_texts(canvas, rsrc);
    }

    pub fn new_request_sender(&self) -> Sender<UI_Request> {
        self.req_tx.clone()
    }

    fn handle_ui_requests(&mut self, rsrc: &mut Resources) {
        let reqs = self.fadeout_text_requests.lock().unwrap().clone();
        for txt in reqs.iter() {
            self.add_fadeout_text(rsrc, &txt);
        }
        self.fadeout_text_requests.lock().unwrap().clear();
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

    fn draw_fadeout_texts(&mut self, canvas: &mut WindowCanvas, rsrc: &mut Resources) {
        let fadeout_time = self.fadeout_time;
        let blend_mode = canvas.blend_mode();
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

        for (i, text) in self.fadeout_texts.iter().enumerate() {
            let texture = rsrc.get_texture_mut(text.texture);
            let TextureQuery { width, height, .. } = texture.query();

            let d = core::time::duration_ratio(&text.time, &fadeout_time);
            let alpha = 255 - (d * d * 255.0f32) as u8;
            texture.set_alpha_mod(alpha);

            let rect = Rect::new(
                Self::FADEOUT_TEXT_PAD_X,
                Self::FADEOUT_TEXT_PAD_Y + (i as i32) * (height as i32 + Self::FADEOUT_TEXT_PAD_Y),
                width,
                height,
            );
            if let Err(msg) = canvas.copy(&texture, None, rect) {
                eprintln!("Error copying texture to window: {}", msg);
            }
        }

        canvas.set_blend_mode(blend_mode);
    }

    pub fn add_fadeout_text(&mut self, resources: &mut Resources, txt: &str) {
        let texture =
            resources.create_font_texture(txt, self.fadeout_text_font, Color::RGB(255, 255, 255));
        self.fadeout_texts.push(Fadeout_Text {
            texture,
            time: Duration::new(0, 0),
        });
    }
}
