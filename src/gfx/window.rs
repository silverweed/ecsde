use crate::core::common::vector::Vec2u;
use sdl2::render::WindowCanvas;

pub fn create_render_canvas(
    video_subsystem: &sdl2::VideoSubsystem,
    target_size: (u32, u32),
    title: &str,
) -> WindowCanvas {
    let window = video_subsystem
        .window(title, target_size.0, target_size.1)
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas
        .set_logical_size(target_size.0, target_size.1)
        .unwrap();
    canvas
}
