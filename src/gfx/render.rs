extern crate sfml;

use self::sfml::graphics as sfgfx;
use self::sfml::graphics::RenderTarget;
use crate::core;
use crate::gfx::window as win;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Render_System {
    window: Rc<RefCell<win::Window>>,
    config: Render_System_Config,
}

impl Render_System {
    pub fn new(window: Rc<RefCell<win::Window>>) -> Self {
        Render_System {
            window,
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

impl core::system::System for Render_System {
    type Config = Render_System_Config;

    fn init(&mut self, cfg: Self::Config) -> core::common::Maybe_Error {
        self.config = cfg;
        Ok(())
    }

    fn update(&mut self, _delta: &std::time::Duration) {
        let win = &mut self.window.borrow_mut().sf_win;
        win.clear(&self.config.clear_color);
        //for &drawable in drawables {
        //self.window.sf_win.draw(drawable);
        //}
        win.display();
    }
}
