extern crate sfml;

use self::sfml::graphics as sfgfx;
use self::sfml::graphics::RenderTarget;
use crate::core;
use crate::gfx::window as win;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Render_System {
    config: Render_System_Config,
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
}

pub struct Render_System_Config {
    pub clear_color: sfgfx::Color,
}

pub struct Render_System_Update_Params {
    pub window: Rc<RefCell<win::Window>>,
}

impl core::system::System for Render_System {
    type Config = Render_System_Config;
    type Update_Params = Render_System_Update_Params;

    fn init(&mut self, cfg: Self::Config) -> core::common::Maybe_Error {
        self.config = cfg;
        Ok(())
    }

    fn update(&mut self, params: Self::Update_Params) {
        let win = &mut params.window.borrow_mut().sf_win;
        win.clear(&self.config.clear_color);
        //for &drawable in drawables {
        //self.window.sf_win.draw(drawable);
        //}
        win.display();
    }
}
