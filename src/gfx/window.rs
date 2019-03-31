use crate::core::common::vector::Vec2u;
use sdl2::video::Window;

pub fn create_render_window(
    video_subsystem: &sdl2::VideoSubsystem,
    target_size: (u32, u32),
    title: &str,
) -> Window {
    video_subsystem
        .window(title, target_size.0, target_size.1)
        .resizable()
        .build()
        .unwrap()
}

// FIXME
pub fn keep_ratio(new_size: &Vec2u, target_size: &Vec2u) -> sdl2::rect::Rect {
    let screen_width = new_size.x as f32 / target_size.x as f32;
    let screen_height = new_size.y as f32 / target_size.y as f32;

    struct FloatRect {
        pub left: f32,
        pub top: f32,
        pub width: f32,
        pub height: f32,
    }
    let mut viewport = FloatRect {
        left: 0f32,
        top: 0f32,
        width: 1f32,
        height: 1f32,
    };
    if screen_width > screen_height {
        viewport.width = screen_height / screen_width;
        viewport.left = 0.5 * (1f32 - viewport.width);
    } else if screen_width < screen_height {
        viewport.height = screen_width / screen_height;
        viewport.top = 0.5 * (1f32 - viewport.height);
    };

    sdl2::rect::Rect::new(
        0,
        0,
        (screen_width * viewport.width) as u32,
        (screen_height * viewport.height) as u32,
    )
}
