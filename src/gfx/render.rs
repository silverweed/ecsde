extern crate sfml;

use self::sfml::graphics as sfgfx;
use self::sfml::graphics::RenderTarget;
use crate::gfx::window as win;

pub struct Renderer {
	window: win::Window,
}

impl Renderer {
	pub fn new(window: win::Window) -> Self {
		Renderer {
			window
		}
	}

	pub fn should_close(&self) -> bool { !self.window.sf_win.is_open() }

	pub fn event_loop(&mut self) {
		win::event_loop(&mut self.window);
	}

	pub fn draw(&mut self, drawables: &[&sfgfx::Drawable]) {
		self.window.sf_win.clear(&sfgfx::Color::BLACK);
		for &drawable in drawables {
			self.window.sf_win.draw(drawable);
		}
		self.window.sf_win.display();
	}
}
