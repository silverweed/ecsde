use super::window::{self, Window_Handle};
use crate::common::colors::Color;
use crate::common::rect::Rect;
use crate::common::transform::Transform2D;
use crate::common::vector::{Vec2f, Vec2i};

#[cfg(feature = "use-sfml")]
mod sfml;

#[cfg(feature = "use-sfml")]
use self::sfml as backend;

pub type Render_Window_Handle = backend::Render_Window_Handle;

pub fn create_render_window(window: Window_Handle) -> Render_Window_Handle {
    backend::create_render_window(window)
}

pub fn clear(window: &mut Render_Window_Handle) {
    backend::clear(window);
}

pub fn display(window: &mut Render_Window_Handle) {
    backend::display(window);
}

pub fn set_clear_color(window: &mut Render_Window_Handle, color: Color) {
    backend::set_clear_color(window, color);
}

pub fn resize_keep_ratio(window: &mut Render_Window_Handle, new_width: u32, new_height: u32) {
    use std::cmp::Ordering;

    if new_width == 0 || new_height == 0 {
        return;
    }

    let (target_width, target_height) = window::get_window_target_size(window);
    assert!(target_width != 0 && target_height != 0);

    let screen_width = new_width as f32 / target_width as f32;
    let screen_height = new_height as f32 / target_height as f32;

    let mut viewport = Rect::new(0.0, 0.0, 1.0, 1.0);
    match screen_width.partial_cmp(&screen_height) {
        Some(Ordering::Greater) => {
            viewport.width = screen_height / screen_width;
            viewport.x = 0.5 * (1.0 - viewport.width);
        }
        Some(Ordering::Less) => {
            viewport.height = screen_width / screen_height;
            viewport.y = 0.5 * (1.0 - viewport.height);
        }
        _ => {}
    }

    let view = Rect::new(0., 0., target_width as f32, target_height as f32);
    backend::set_viewport(window, &viewport, &view);
}

pub fn unproject_screen_pos(
    screen_pos: Vec2i,
    window: &Render_Window_Handle,
    camera: &Transform2D,
) -> Vec2f {
    backend::raw_unproject_screen_pos(screen_pos, window, camera)
}

pub fn project_world_pos(
    world_pos: Vec2f,
    window: &Render_Window_Handle,
    camera: &Transform2D,
) -> Vec2i {
    backend::raw_project_world_pos(world_pos, window, camera)
}

pub fn mouse_pos_in_world(window: &Render_Window_Handle, camera: &Transform2D) -> Vec2f {
    unproject_screen_pos(window::raw_mouse_pos_in_window(window), window, camera)
}
