use super::paint_props::Paint_Properties;
use crate::common::colors::Color;
use crate::common::rect::{Rect, Rectf};
use crate::common::shapes::Circle;
use crate::common::transform::Transform2D;
use crate::common::vector::Vec2f;
use crate::gfx::window::Window_Handle;
use std::convert::Into;

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
pub fn fill_color_rect<R, P>(window: &mut Window_Handle, paint_props: P, rect: R)
where
    R: Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
    P: Into<Paint_Properties>,
{
    backend::fill_color_rect(window, &paint_props.into(), rect);
}

/// Draws a color-filled rectangle in world space
pub fn fill_color_rect_ws<R, P>(
    window: &mut Window_Handle,
    paint_props: P,
    rect: R,
    transform: &Transform2D,
    camera: &Transform2D,
) where
    R: Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
    P: Into<Paint_Properties>,
{
    backend::fill_color_rect_ws(window, &paint_props.into(), rect, transform, camera);
}

/// Draws a color-filled circle in world space
pub fn fill_color_circle_ws<P>(
    window: &mut Window_Handle,
    paint_props: P,
    circle: Circle,
    camera: &Transform2D,
) where
    P: Into<Paint_Properties>,
{
    backend::fill_color_circle_ws(window, &paint_props.into(), circle, camera);
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

// @Cleanup: ideally we'd want to pass a &Text, not a &mut Text.
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

pub fn set_text_paint_props<P>(text: &mut Text, paint_props: P)
where
    P: Into<Paint_Properties>,
{
    backend::set_text_paint_props(text, paint_props.into());
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

pub fn add_vertex(vbuf: &mut Vertex_Buffer, v: &Vertex) {
    backend::add_vertex(vbuf, v);
}

pub fn new_vertex(pos: Vec2f, col: Color, tex_coords: Vec2f) -> Vertex {
    backend::new_vertex(pos, col, tex_coords)
}

pub fn render_vbuf(window: &mut Window_Handle, vbuf: &Vertex_Buffer, transform: &Transform2D) {
    backend::render_vbuf(window, vbuf, transform);
}

pub fn render_vbuf_ws(
    window: &mut Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    backend::render_vbuf_ws(window, vbuf, transform, camera);
}

pub fn start_draw_linestrip(n_vertices: usize) -> Vertex_Buffer {
    backend::start_draw_linestrip(n_vertices)
}

pub fn render_line(window: &mut Window_Handle, start: &Vertex, end: &Vertex) {
    backend::render_line(window, start, end);
}
