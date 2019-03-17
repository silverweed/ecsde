extern crate sfml;

use self::sfml::graphics as sfgfx;
use self::sfml::system as sfsys;
use self::sfml::window as sfwin;

pub struct Window {
    pub sf_win: sfgfx::RenderWindow,
    pub target_size: sfsys::Vector2u,
}

pub fn create_render_window(target_size: (u32, u32), title: &str) -> Window {
    let mut window = sfgfx::RenderWindow::new(
        sfwin::VideoMode::new(
            target_size.0,
            target_size.1,
            sfwin::VideoMode::desktop_mode().bits_per_pixel,
        ),
        title,
        sfwin::Style::CLOSE | sfwin::Style::RESIZE,
        &Default::default(),
    );

    window.set_vertical_sync_enabled(true);
    window.set_framerate_limit(60);

    Window {
        sf_win: window,
        target_size: sfsys::Vector2u::new(target_size.0, target_size.1),
    }
}

pub fn keep_ratio(new_size: &sfsys::Vector2u, target_size: &sfsys::Vector2u) -> sfgfx::View {
    let screen_width = new_size.x as f32 / target_size.x as f32;
    let screen_height = new_size.y as f32 / target_size.y as f32;

    let mut viewport = sfgfx::FloatRect::new(0f32, 0f32, 1f32, 1f32);
    if screen_width > screen_height {
        viewport.width = screen_height / screen_width;
        viewport.left = 0.5 * (1f32 - viewport.width);
    } else if screen_width < screen_height {
        viewport.height = screen_width / screen_height;
        viewport.top = 0.5 * (1f32 - viewport.height);
    };

    let mut view = sfgfx::View::from_rect(&sfgfx::FloatRect::new(
        0f32,
        0f32,
        target_size.x as f32,
        target_size.y as f32,
    ));
    view.set_viewport(&viewport);
    view
}
