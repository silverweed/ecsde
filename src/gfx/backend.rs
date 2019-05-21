#[cfg(feature = "gfx_sdl")]
mod sdl;

use crate::core::common::colors::Color;
use crate::core::common::rect::Rect;
use crate::ecs::components::transform::C_Transform2D;
use crate::gfx::render::Sprite;
use crate::resources::{Resources, Texture_Handle};

// ------------- Backend: SDL
#[cfg(feature = "gfx_sdl")]
pub type Create_Render_Window_Args = sdl::window::Create_Render_Window_Args;
#[cfg(feature = "gfx_sdl")]
pub type Window_Handle = sdl::window::Window_Handle;
#[cfg(feature = "gfx_sdl")]
pub type Blend_Mode = sdl::render::Blend_Mode;
#[cfg(feature = "gfx_sdl")]
pub type Texture<'a> = sdl::render::Texture<'a>;

#[cfg(feature = "gfx_sdl")]
pub fn create_render_window(
    video_subsystem: &Create_Render_Window_Args,
    target_size: (u32, u32),
    title: &str,
) -> Window_Handle {
    sdl::window::create_render_window(video_subsystem, target_size, title)
}

#[cfg(feature = "gfx_sdl")]
pub fn set_clear_color(window: &mut Window_Handle, color: Color) {
    sdl::window::set_clear_color(window, color);
}

#[cfg(feature = "gfx_sdl")]
pub fn clear(window: &mut Window_Handle) {
    sdl::window::clear(window);
}

#[cfg(feature = "gfx_sdl")]
pub fn display(window: &mut Window_Handle) {
    sdl::window::display(window);
}

#[cfg(feature = "gfx_sdl")]
pub fn render_sprite(
    window: &mut Window_Handle,
    sprite: &Sprite<'_>,
    transform: &C_Transform2D,
    camera: &C_Transform2D,
) {
    sdl::render::render_sprite(window, sprite, transform, camera);
}

#[cfg(feature = "gfx_sdl")]
pub fn render_texture(window: &mut Window_Handle, texture: &Texture<'_>, rect: Rect) {
    sdl::render::render_texture(window, texture, rect);
}

#[cfg(feature = "gfx_sdl")]
pub fn get_blend_mode(window: &Window_Handle) -> Blend_Mode {
    sdl::render::get_blend_mode(window)
}

#[cfg(feature = "gfx_sdl")]
pub fn set_blend_mode(window: &mut Window_Handle, blend_mode: Blend_Mode) {
    sdl::render::set_blend_mode(window, blend_mode);
}

#[cfg(feature = "gfx_sdl")]
pub fn get_texture_size(texture: &Texture<'_>) -> (u32, u32) {
    sdl::render::get_texture_size(texture)
}
