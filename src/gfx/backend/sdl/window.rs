use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use crate::core::common::colors::Color;

pub type Create_Render_Window_Args = sdl2::VideoSubsystem;
pub type Window_Handle = WindowCanvas;

pub fn create_render_window(
    video_subsystem: &Create_Render_Window_Args,
    target_size: (u32, u32),
    title: &str,
) -> Window_Handle {
    let window = video_subsystem
        .window(title, target_size.0, target_size.1)
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .unwrap();
    canvas
        .set_logical_size(target_size.0, target_size.1)
        .unwrap();
    canvas
}

pub fn set_clear_color(window: &mut WindowCanvas, color: Color) {
    window.set_draw_color(color);
}

pub fn clear(window: &mut WindowCanvas) {
    let (out_x, out_y) = window.output_size().unwrap();
    window.fill_rect(Rect::new(0, 0, out_x, out_y)).unwrap();
}
