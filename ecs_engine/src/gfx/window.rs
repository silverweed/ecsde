use crate::core::common::colors::Color;
use crate::core::common::rect::Rect;

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

pub fn get_framerate_limit(window: &Window_Handle) -> u32 {
    backend::get_framerate_limit(window)
}

pub fn set_framerate_limit(window: &mut Window_Handle, limit: u32) {
    backend::set_framerate_limit(window, limit);
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

    let (target_width, target_height) = get_window_target_size(window);
    let screen_width = new_width as f32 / target_width as f32;
    let screen_height = new_height as f32 / target_height as f32;

    // @Robustness: what do we do if width or height are zero?
    debug_assert!(screen_width.is_normal());
    debug_assert!(screen_height.is_normal());

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

    backend::set_viewport(window, &viewport);
}

pub fn get_window_target_size(window: &Window_Handle) -> (u32, u32) {
    backend::get_window_target_size(window)
}
