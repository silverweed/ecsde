use crate::common::colors::Color;
use crate::common::rect::Rectf;
use crate::common::transform::Transform2D;
use crate::common::vector::{Vec2f, Vec2i};
use crate::gfx::window::Window_Handle;

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

pub fn create_render_window(window: Window_Handle) -> Render_Window_Handle {
    Render_Window_Handle { window }
}

pub fn set_clear_color(_window: &mut Render_Window_Handle, _color: Color) {}

pub fn clear(_window: &mut Render_Window_Handle) {}

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
