#[cfg(feature = "use-sdl")]
mod sdl;
#[cfg(feature = "use-sfml")]
mod sfml;

use crate::core::common::colors::Color;
use crate::core::common::rect::Rect;
use crate::ecs::components::transform::C_Transform2D;

// -----------------------------------------------------------------------------
// ----------------------------- Backend: SDL ----------------------------------
// -----------------------------------------------------------------------------
#[cfg(feature = "use-sdl")]
pub type Create_Render_Window_Args = sdl::window::Create_Render_Window_Args;
#[cfg(feature = "use-sdl")]
pub type Window_Handle = sdl::window::Window_Handle;
#[cfg(feature = "use-sdl")]
pub type Blend_Mode = sdl::render::Blend_Mode;
#[cfg(feature = "use-sdl")]
pub type Texture<'a> = sdl::render::Texture<'a>;
#[cfg(feature = "use-sdl")]
pub type Sprite<'a> = sdl::render::Sprite<'a>;
#[cfg(feature = "use-sdl")]
pub type Text<'a> = sdl::render::Text<'a>;
#[cfg(feature = "use-sdl")]
pub type Font<'a> = sdl::render::Font<'a>;

#[cfg(feature = "use-sdl")]
pub fn create_render_window(
    video_subsystem: &Create_Render_Window_Args,
    target_size: (u32, u32),
    title: &str,
) -> Window_Handle {
    sdl::window::create_render_window(video_subsystem, target_size, title)
}

#[cfg(feature = "use-sdl")]
pub fn set_clear_color(window: &mut Window_Handle, color: Color) {
    sdl::window::set_clear_color(window, color);
}

#[cfg(feature = "use-sdl")]
pub fn clear(window: &mut Window_Handle) {
    sdl::window::clear(window);
}

#[cfg(feature = "use-sdl")]
pub fn display(window: &mut Window_Handle) {
    sdl::window::display(window);
}

#[cfg(feature = "use-sdl")]
pub fn create_sprite<'a>(texture: &'a Texture<'a>, rect: Rect) -> Sprite<'a> {
    sdl::render::create_sprite(texture, rect)
}

#[cfg(feature = "use-sdl")]
pub fn render_sprite(
    window: &mut Window_Handle,
    sprite: &Sprite<'_>,
    transform: &C_Transform2D,
    camera: &C_Transform2D,
) {
    sdl::render::render_sprite(window, sprite, transform, camera);
}

#[cfg(feature = "use-sdl")]
pub fn render_texture(window: &mut Window_Handle, texture: &Texture<'_>, rect: Rect) {
    sdl::render::render_texture(window, texture, rect);
}

#[cfg(feature = "use-sdl")]
pub fn render_text(window: &mut Window_Handle, text: &Text<'_>) {
    sdl::render::render_text(window, text);
}

#[cfg(feature = "use-sdl")]
pub fn get_blend_mode(window: &Window_Handle) -> Blend_Mode {
    sdl::render::get_blend_mode(window)
}

#[cfg(feature = "use-sdl")]
pub fn set_blend_mode(window: &mut Window_Handle, blend_mode: Blend_Mode) {
    sdl::render::set_blend_mode(window, blend_mode);
}

#[cfg(feature = "use-sdl")]
pub fn get_texture_size(texture: &Texture<'_>) -> (u32, u32) {
    sdl::render::get_texture_size(texture)
}

// -----------------------------------------------------------------------------
// ----------------------------- Backend: SFML ---------------------------------
// -----------------------------------------------------------------------------
#[cfg(feature = "use-sfml")]
pub type Create_Render_Window_Args = sfml::window::Create_Render_Window_Args;
#[cfg(feature = "use-sfml")]
pub type Window_Handle = sfml::window::Window_Handle;
#[cfg(feature = "use-sfml")]
pub type Blend_Mode = sfml::render::Blend_Mode;
#[cfg(feature = "use-sfml")]
pub type Texture<'a> = sfml::render::Texture<'a>;
#[cfg(feature = "use-sfml")]
pub type Sprite<'a> = sfml::render::Sprite<'a>;
#[cfg(feature = "use-sfml")]
pub type Text<'a> = sfml::render::Text<'a>;
#[cfg(feature = "use-sfml")]
pub type Font<'a> = sfml::render::Font<'a>;

#[cfg(feature = "use-sfml")]
pub fn create_render_window(
    video_subsystem: &Create_Render_Window_Args,
    target_size: (u32, u32),
    title: &str,
) -> Window_Handle {
    sfml::window::create_render_window(video_subsystem, target_size, title)
}

#[cfg(feature = "use-sfml")]
pub fn set_clear_color(window: &mut Window_Handle, color: Color) {
    sfml::window::set_clear_color(window, color);
}

#[cfg(feature = "use-sfml")]
pub fn clear(window: &mut Window_Handle) {
    sfml::window::clear(window);
}

#[cfg(feature = "use-sfml")]
pub fn display(window: &mut Window_Handle) {
    sfml::window::display(window);
}
#[cfg(feature = "use-sfml")]
pub fn resize_keep_ratio(window: &mut Window_Handle, new_width: u32, new_height: u32) {
    sfml::window::resize_keep_ratio(window, new_width, new_height);
}

#[cfg(feature = "use-sfml")]
pub fn create_sprite<'a>(texture: &'a Texture<'a>, rect: Rect) -> Sprite<'a> {
    sfml::render::create_sprite(texture, rect)
}

#[cfg(feature = "use-sfml")]
pub fn render_sprite(
    window: &mut Window_Handle,
    sprite: &Sprite<'_>,
    transform: &C_Transform2D,
    camera: &C_Transform2D,
) {
    sfml::render::render_sprite(window, sprite, transform, camera);
}

#[cfg(feature = "use-sfml")]
pub fn render_texture(window: &mut Window_Handle, texture: &Texture<'_>, rect: Rect) {
    sfml::render::render_texture(window, texture, rect);
}

#[cfg(feature = "use-sfml")]
pub fn render_text(window: &mut Window_Handle, text: &Text<'_>) {
    sfml::render::render_text(window, text);
}

#[cfg(feature = "use-sfml")]
pub fn get_blend_mode(window: &Window_Handle) -> Blend_Mode {
    sfml::render::get_blend_mode(window)
}

#[cfg(feature = "use-sfml")]
pub fn set_blend_mode(window: &mut Window_Handle, blend_mode: Blend_Mode) {
    sfml::render::set_blend_mode(window, blend_mode);
}

#[cfg(feature = "use-sfml")]
pub fn get_texture_size(texture: &Texture<'_>) -> (u32, u32) {
    sfml::render::get_texture_size(texture)
}
