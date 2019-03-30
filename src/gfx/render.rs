use crate::core;
use sdl2::pixels::Color;
use sdl2::render::Canvas;

pub struct Render_System {
    config: Render_System_Config,
}

pub struct Render_System_Config {
    pub clear_color: Color,
}

impl Render_System {
    pub fn new() -> Self {
        Render_System {
            config: Self::default_config(),
        }
    }

    fn default_config() -> Render_System_Config {
        Render_System_Config {
            clear_color: Color::RGB(0, 0, 0),
        }
    }

    pub fn init(&mut self, cfg: Render_System_Config) -> core::common::Maybe_Error {
        self.config = cfg;
        Ok(())
    }

    pub fn update(&mut self, canvas: &mut Canvas<sdl2::video::Window>) {
        canvas.set_draw_color(self.config.clear_color);
        canvas.clear();
        canvas.present();
    }
}
