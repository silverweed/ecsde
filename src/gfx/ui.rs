use crate::resources::{self, Texture_Handle};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::ttf::{self, Font};
use std::vec::Vec;

pub struct UI_System {
    texts: Vec<Texture_Handle>,
}

impl UI_System {
    pub fn new() -> UI_System {
        UI_System { texts: vec![] }
    }

    pub fn update(&mut self, canvas: &mut sdl2::render::WindowCanvas, rsrc: &resources::Resources) {
        let rect = Rect::new(0, 0, 400, 100);
        for tex_handle in self.texts.iter() {
            let texture = rsrc.get_texture(*tex_handle);
            if let Err(msg) = canvas.copy_ex(&texture, None, rect, 0.0, None, false, false) {
                eprintln!("Error copying texture to window: {}", msg);
            }
        }
    }

    pub fn add_fadeout_text(
        &mut self,
        resources: &mut resources::Resources,
        font: resources::Font_Handle,
        txt: &str,
        fadeout_time: u32,
    ) {
        let texture =
            resources.create_font_texture("Hello sailor!", font, Color::RGB(255, 255, 255));
        self.texts.push(texture);

        // @Incomplete: add fadeout logic
    }
}
