extern crate sfml;

use self::sfml::window as sfwin;
use self::sfml::graphics as sfgfx;
use self::sfwin::Key;

pub fn create_render_window<V: Into<sfwin::VideoMode>>(video_mode: V, title: &str) -> sfgfx::RenderWindow {
	let mut window = sfgfx::RenderWindow::new(
		video_mode,
		title,
		sfwin::Style::CLOSE,
		&Default::default());

	window.set_framerate_limit(60);

	window
}

pub fn event_loop(window: &mut sfgfx::RenderWindow) {
	while let Some(event) = window.poll_event() {
		match event {
			sfwin::Event::Closed => { window.close(); },
			sfwin::Event::KeyPressed {
				code,
				..
			} => match code {
				Key::Q => { window.close(); },
				_ => ()
			},
			_ => ()
		}
	}
}
