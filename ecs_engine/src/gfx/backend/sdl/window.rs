use crate::core::common::colors::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

pub type Create_Render_Window_Args = sdl2::VideoSubsystem;

pub struct Window_Handle(WindowCanvas);

impl std::ops::Deref for Window_Handle {
    type Target = WindowCanvas;

    fn deref(&self) -> &WindowCanvas {
        &self.0
    }
}

impl std::ops::DerefMut for Window_Handle {
    fn deref_mut(&mut self) -> &mut WindowCanvas {
        &mut self.0
    }
}

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
    Window_Handle(canvas)
}

pub fn set_clear_color(window: &mut WindowCanvas, color: Color) {
    window.set_draw_color(color);
}

pub fn clear(window: &mut WindowCanvas) {
    let (out_x, out_y) = window.output_size().unwrap();
    window.fill_rect(Rect::new(0, 0, out_x, out_y)).unwrap();
}

pub fn display(window: &mut WindowCanvas) {
    window.present();
}
