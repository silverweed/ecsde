use super::backend;
use crate::core::common::colors::Color;

pub type Window_Handle = backend::Window_Handle;
pub type Create_Render_Window_Args = backend::Create_Render_Window_Args;

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
