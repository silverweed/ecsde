use super::paint_props::Paint_Properties;
use crate::core::common::colors::Color;
use crate::core::common::rect::{Rect, Rectf};
use crate::core::common::shapes::Circle;
use crate::core::common::transform::Transform2D;
use crate::core::common::vector::Vec2f;
use crate::gfx::window::Window_Handle;

#[cfg(feature = "use-sfml")]
mod sfml;

#[cfg(feature = "use-sfml")]
use self::sfml as backend;

pub type Texture<'a> = backend::Texture<'a>;
pub type Sprite<'a> = backend::Sprite<'a>;
pub type Text<'a> = backend::Text<'a>;
pub type Font<'a> = backend::Font<'a>;
pub type Vertex_Buffer = backend::Vertex_Buffer;
pub type Vertex = backend::Vertex;

pub fn create_sprite<'a>(texture: &'a Texture<'a>, rect: &Rect<i32>) -> Sprite<'a> {
    backend::create_sprite(texture, rect)
}

pub fn render_sprite(
    window: &mut Window_Handle,
    sprite: &mut Sprite,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    backend::render_sprite(window, sprite, transform, camera);
}

pub fn set_sprite_modulate(sprite: &mut Sprite, modulate: Color) {
    backend::set_sprite_modulate(sprite, modulate);
}

pub fn sprite_global_bounds(sprite: &Sprite) -> Rect<f32> {
    backend::sprite_global_bounds(sprite)
}

/// Draws a color-filled rectangle in screen space
pub fn fill_color_rect<T>(window: &mut Window_Handle, paint_props: &Paint_Properties, rect: T)
where
    T: std::convert::Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
{
    backend::fill_color_rect(window, paint_props, rect);
}

/// Draws a color-filled rectangle in world space
pub fn fill_color_rect_ws<T>(
    window: &mut Window_Handle,
    paint_props: &Paint_Properties,
    rect: T,
    transform: &Transform2D,
    camera: &Transform2D,
) where
    T: std::convert::Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
{
    backend::fill_color_rect_ws(window, paint_props, rect, transform, camera);
}

/// Draws a color-filled circle in world space
pub fn fill_color_circle_ws(
    window: &mut Window_Handle,
    paint_props: &Paint_Properties,
    circle: Circle,
    camera: &Transform2D,
) {
    backend::fill_color_circle_ws(window, paint_props, circle, camera);
}

pub fn render_texture(window: &mut Window_Handle, texture: &Texture<'_>, rect: Rect<i32>) {
    backend::render_texture(window, texture, rect);
}

pub fn get_texture_size(texture: &Texture<'_>) -> (u32, u32) {
    backend::get_texture_size(texture)
}

pub fn create_text<'a>(string: &str, font: &'a Font, size: u16) -> Text<'a> {
    backend::create_text(string, font, size)
}

pub fn render_text(window: &mut Window_Handle, text: &mut Text, screen_pos: Vec2f) {
    backend::render_text(window, text, screen_pos);
}

pub fn render_text_ws(
    window: &mut Window_Handle,
    text: &Text,
    world_transform: &Transform2D,
    camera: &Transform2D,
) {
    backend::render_text_ws(window, text, world_transform, camera);
}

pub fn set_text_fill_color(text: &mut Text, color: Color) {
    backend::set_text_fill_color(text, color);
}

pub fn get_text_local_bounds(text: &Text) -> Rectf {
    backend::get_text_local_bounds(text)
}

pub fn start_draw_quads(n_quads: usize) -> Vertex_Buffer {
    backend::start_draw_quads(n_quads)
}

pub fn add_quad(vbuf: &mut Vertex_Buffer, v1: &Vertex, v2: &Vertex, v3: &Vertex, v4: &Vertex) {
    backend::add_quad(vbuf, v1, v2, v3, v4);
}

pub fn new_vertex(pos: Vec2f, col: Color, tex_coords: Vec2f) -> Vertex {
    backend::new_vertex(pos, col, tex_coords)
}

pub fn render_vbuf_ws(
    window: &mut Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    backend::render_vbuf_ws(window, vbuf, transform, camera);
}
