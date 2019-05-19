use super::backend;
use crate::core::common::transform::C_Transform2D;
use crate::core::common::rect::Rect;
use crate::gfx;
use crate::gfx::window::Window_Handle;
use crate::resources;

pub type Blend_Mode = backend::Blend_Mode;
pub type Texture<'a> = backend::Texture<'a>;

pub struct Sprite<'a> {
    pub texture: &'a Texture<'a>,
    pub rect: Rect,
}

pub fn get_blend_mode(window: &Window_Handle) -> Blend_Mode {
    backend::get_blend_mode(window)
}

pub fn set_blend_mode(window: &mut Window_Handle, blend_mode: Blend_Mode) {
    backend::set_blend_mode(window, blend_mode);
}

pub fn render_sprite(
    window: &mut Window_Handle,
    sprite: &Sprite<'_>,
    transform: &C_Transform2D,
    camera: &C_Transform2D)
{
    backend::render_sprite(window, sprite, transform, camera);
}

pub fn render_texture(window: &mut Window_Handle, texture: &Texture<'_>, rect: Rect) {
    backend::render_texture(window, texture, rect);
}

pub fn get_texture_size(texture: &Texture<'_>) -> (u32, u32) {
    backend::get_texture_size(texture)
}
