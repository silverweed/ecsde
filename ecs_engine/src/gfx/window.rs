use crate::common::colors::Color;
use crate::common::rect::Rect;
use crate::common::transform::Transform2D;
use crate::common::vector::{Vec2f, Vec2i, Vec2u};

#[cfg(feature = "use-sfml")]
mod sfml;

#[cfg(feature = "use-sfml")]
use self::sfml as backend;

pub type Window_Handle = backend::Window_Handle;
pub type Create_Render_Window_Args = backend::Create_Render_Window_Args;
pub type Blend_Mode = backend::Blend_Mode;

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn create_render_window(
    args: &Create_Render_Window_Args,
    target_size: (u32, u32),
    title: &str,
) -> Window_Handle {
    backend::create_render_window(args, target_size, title)
}

pub fn get_blend_mode(window: &Window_Handle) -> Blend_Mode {
    backend::get_blend_mode(window)
}

pub fn set_blend_mode(window: &mut Window_Handle, blend_mode: Blend_Mode) {
    backend::set_blend_mode(window, blend_mode);
}

pub fn destroy_render_window(window: &mut Window_Handle) {
    backend::destroy_render_window(window);
}

pub fn set_clear_color(window: &mut Window_Handle, color: Color) {
    backend::set_clear_color(window, color);
}

pub fn has_vsync(window: &Window_Handle) -> bool {
    backend::has_vsync(window)
}

pub fn set_vsync(window: &mut Window_Handle, vsync: bool) {
    backend::set_vsync(window, vsync);
}

pub fn clear(window: &mut Window_Handle) {
    backend::clear(window);
}

pub fn display(window: &mut Window_Handle) {
    backend::display(window);
}

pub fn resize_keep_ratio(window: &mut Window_Handle, new_width: u32, new_height: u32) {
    use std::cmp::Ordering;

    if new_width == 0 || new_height == 0 {
        return;
    }

    let (target_width, target_height) = get_window_target_size(window);
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

pub fn get_window_target_size(window: &Window_Handle) -> (u32, u32) {
    backend::get_window_target_size(window)
}

pub fn get_window_real_size(window: &Window_Handle) -> (u32, u32) {
    backend::get_window_real_size(window)
}

// Returns the mouse position relative to the actual window,
// without taking the target size into account (so if the window aspect ratio
// does not match with the target ratio, the result does not take "black bands" into account).
// Use this when you want to unproject mouse coordinates!
pub fn raw_mouse_pos_in_window(window: &Window_Handle) -> Vec2i {
    backend::raw_mouse_pos_in_window(window)
}

pub fn mouse_pos_in_world(window: &Window_Handle, camera: &Transform2D) -> Vec2f {
    unproject_screen_pos(raw_mouse_pos_in_window(window), window, camera)
}

pub fn mouse_pos_in_window(window: &Window_Handle) -> Vec2i {
    let v = Vec2f::from(backend::raw_mouse_pos_in_window(window));

    let ts: Vec2u = backend::get_window_target_size(window).into();
    let target_ratio = ts.y as f32 / ts.x as f32;

    let rs: Vec2u = backend::get_window_real_size(window).into();
    let real_ratio = rs.y as f32 / rs.x as f32;

    let ratio = Vec2f::from(rs) / Vec2f::from(ts);

    let x;
    let y;
    if real_ratio <= target_ratio {
        let delta = (rs.x as f32 - rs.y as f32 / target_ratio) * 0.5;
        y = v.y / ratio.y;
        x = (v.x - delta) / ratio.y;
    } else {
        let delta = (rs.y as f32 - rs.x as f32 * target_ratio) * 0.5;
        x = v.x / ratio.x;
        y = (v.y - delta) / ratio.x;
    }

    Vec2i::new(x as _, y as _)
}

pub fn unproject_screen_pos(
    screen_pos: Vec2i,
    window: &Window_Handle,
    camera: &Transform2D,
) -> Vec2f {
    backend::raw_unproject_screen_pos(screen_pos, window, camera)
}

pub fn project_world_pos(world_pos: Vec2f, window: &Window_Handle, camera: &Transform2D) -> Vec2i {
    backend::raw_project_world_pos(world_pos, window, camera)
}

pub fn set_key_repeat_enabled(window: &mut Window_Handle, enabled: bool) {
    backend::set_key_repeat_enabled(window, enabled);
}
