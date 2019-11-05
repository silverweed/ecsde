use super::backend;
use crate::core::common::colors::Color;

pub type Window_Handle = backend::Window_Handle;
pub type Create_Render_Window_Args = backend::Create_Render_Window_Args;

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn create_render_window(
    args: &Create_Render_Window_Args,
    target_size: (u32, u32),
    title: &str,
) -> Window_Handle {
    backend::create_render_window(args, target_size, title)
}

pub fn set_clear_color(window: &mut Window_Handle, color: Color) {
    backend::set_clear_color(window, color);
}

pub fn clear(window: &mut Window_Handle) {
    backend::clear(window);
}

pub fn display(window: &mut Window_Handle) {
    backend::display(window);
}

pub fn resize_keep_ratio(window: &mut Window_Handle, new_width: u32, new_height: u32) {
    backend::resize_keep_ratio(window, new_width, new_height);
}

pub fn get_window_target_size(window: &Window_Handle) -> (u32, u32) {
    backend::get_window_target_size(window)
}
