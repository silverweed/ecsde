use inle_common::colors::Color;
use inle_math::rect::Rectf;
use inle_math::transform::Transform2D;
use inle_math::vector::{Vec2f, Vec2i};
use inle_win::window::Window_Handle;

pub struct Render_Window_Handle {
    window: Window_Handle,
}

impl AsRef<Window_Handle> for Render_Window_Handle {
    fn as_ref(&self) -> &Window_Handle {
        &self.window
    }
}

impl AsMut<Window_Handle> for Render_Window_Handle {
    fn as_mut(&mut self) -> &mut Window_Handle {
        &mut self.window
    }
}

pub fn create_render_window(mut window: Window_Handle) -> Render_Window_Handle {
    gl::load_with(|symbol| inle_win::window::get_gl_handle(&mut window, symbol));
    Render_Window_Handle { window }
}

pub fn set_clear_color(_window: &mut Render_Window_Handle, color: Color) {
    unsafe {
        gl::ClearColor(
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        );
    }
}

pub fn clear(_window: &mut Render_Window_Handle) {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

pub fn set_viewport(_window: &mut Render_Window_Handle, _viewport: &Rectf, _view_rect: &Rectf) {}

pub fn raw_unproject_screen_pos(
    _screen_pos: Vec2i,
    _window: &Render_Window_Handle,
    _camera: &Transform2D,
) -> Vec2f {
    Vec2f::default()
}

pub fn raw_project_world_pos(
    _world_pos: Vec2f,
    _window: &Render_Window_Handle,
    _camera: &Transform2D,
) -> Vec2i {
    Vec2i::default()
}
