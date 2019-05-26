use super::backend;
use crate::core::common::rect::Rect;
use crate::ecs::components::transform::C_Transform2D;
use crate::gfx::window::Window_Handle;

pub type Blend_Mode = backend::Blend_Mode;
pub type Texture<'a> = backend::Texture<'a>;
pub type Sprite<'a> = backend::Sprite<'a>;
pub type Text<'a> = backend::Text<'a>;
pub type Font<'a> = backend::Font<'a>;

pub fn get_blend_mode(window: &Window_Handle) -> Blend_Mode {
    backend::get_blend_mode(window)
}

pub fn set_blend_mode(window: &mut Window_Handle, blend_mode: Blend_Mode) {
    backend::set_blend_mode(window, blend_mode);
}

pub fn create_sprite<'a>(texture: &'a Texture<'a>, rect: Rect) -> Sprite<'a> {
    backend::create_sprite(texture, rect)
}

pub fn render_sprite(
    window: &mut Window_Handle,
    sprite: &Sprite<'_>,
    transform: &C_Transform2D,
    camera: &C_Transform2D,
) {
    backend::render_sprite(window, sprite, transform, camera);
}

pub fn render_texture(window: &mut Window_Handle, texture: &Texture<'_>, rect: Rect) {
    backend::render_texture(window, texture, rect);
}

pub fn get_texture_size(texture: &Texture<'_>) -> (u32, u32) {
    backend::get_texture_size(texture)
}

pub fn render_text(window: &mut Window_Handle, text: &Text) {
    backend::render_text(window, text);
}
