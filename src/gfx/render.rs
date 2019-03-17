extern crate sfml;

use self::sfml::graphics as sfgfx;
use self::sfml::graphics::RenderTarget;
use crate::core;

pub struct Render_System {
    config: Render_System_Config,
}

pub struct Render_System_Config {
    pub clear_color: sfgfx::Color,
}

impl Render_System {
    pub fn new() -> Self {
        Render_System {
            config: Self::default_config(),
        }
    }

    fn default_config() -> Render_System_Config {
        Render_System_Config {
            clear_color: sfgfx::Color::BLACK,
        }
    }

    pub fn init(&mut self, cfg: Render_System_Config) -> core::common::Maybe_Error {
        self.config = cfg;
        Ok(())
    }

    pub fn update(&mut self, window: &mut sfgfx::RenderWindow) {
        window.clear(&self.config.clear_color);
        window.display();
    }
}
