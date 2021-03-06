use crate::material::Material;
use inle_common::colors::Color;
use inle_common::paint_props::Paint_Properties;
use inle_gfx_backend::render::backend;
use inle_gfx_backend::render_window::Render_Window_Handle;
use inle_math::rect::Rect;
use inle_math::shapes::Circle;
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;

pub use inle_gfx_backend::render::{Primitive_Type, Render_Extra_Params};

pub mod batcher;

pub type Z_Index = i8;
pub type Font<'a> = backend::Font<'a>;
pub type Image = backend::Image;
pub type Shader<'a> = backend::Shader<'a>;
pub type Text<'a> = backend::Text<'a>;
pub type Texture<'a> = backend::Texture<'a>;
pub type Vertex_Buffer = backend::Vertex_Buffer;
pub type Vertex = backend::Vertex;

//////////////////////////// DRAWING //////////////////////////////////

/// Draws a color-filled rectangle in screen space
pub fn render_rect<R, P>(window: &mut Render_Window_Handle, rect: R, paint_props: P)
where
    R: Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
    P: Into<Paint_Properties>,
{
    trace!("render_rect");
    let paint_props = paint_props.into();
    backend::fill_color_rect(window, &paint_props, rect);
}

/// Draws a color-filled rectangle in world space
pub fn render_rect_ws<R, P>(
    window: &mut Render_Window_Handle,
    rect: R,
    paint_props: P,
    transform: &Transform2D,
    camera: &Transform2D,
) where
    R: Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
    P: Into<Paint_Properties>,
{
    trace!("render_rect_ws");
    let paint_props = paint_props.into();
    backend::fill_color_rect_ws(window, &paint_props, rect, transform, camera);
}

pub fn render_circle<P>(window: &mut Render_Window_Handle, circle: Circle, paint_props: P)
where
    P: Into<Paint_Properties>,
{
    trace!("render_circle");
    let paint_props = paint_props.into();
    backend::fill_color_circle(window, &paint_props, circle);
}

/// Draws a color-filled circle in world space
pub fn render_circle_ws<P>(
    window: &mut Render_Window_Handle,
    circle: Circle,
    paint_props: P,
    camera: &Transform2D,
) where
    P: Into<Paint_Properties>,
{
    trace!("render_circle_ws");
    let paint_props = paint_props.into();
    backend::fill_color_circle_ws(window, &paint_props, circle, camera);
}

pub fn render_texture_ws(
    batches: &mut batcher::Batches,
    material: Material,
    tex_rect: &Rect<i32>,
    color: Color,
    transform: &Transform2D,
    z_index: Z_Index,
) {
    trace!("render_texture_ws");
    batcher::add_texture_ws(batches, material, tex_rect, color, transform, z_index);
}

pub fn render_text<P>(
    window: &mut Render_Window_Handle,
    text: &mut Text<'_>,
    paint_props: P,
    screen_pos: Vec2f,
) where
    P: Into<Paint_Properties>,
{
    trace!("render_text");
    backend::render_text(window, text, &paint_props.into(), screen_pos);
}

pub fn render_text_ws<P>(
    window: &mut Render_Window_Handle,
    text: &mut Text<'_>,
    paint_props: P,
    world_transform: &Transform2D,
    camera: &Transform2D,
) where
    P: Into<Paint_Properties>,
{
    trace!("render_text_ws");
    backend::render_text_ws(window, text, &paint_props.into(), world_transform, camera);
}

pub fn render_vbuf(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
) {
    trace!("render_vbuf");
    backend::render_vbuf(window, vbuf, transform);
}

pub fn render_vbuf_ws(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    trace!("render_vbuf_ws");
    backend::render_vbuf_ws(window, vbuf, transform, camera);
}

pub fn render_vbuf_ws_ex(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
    camera: &Transform2D,
    extra_params: Render_Extra_Params,
) {
    trace!("render_vbuf_ws");
    backend::render_vbuf_ws_ex(window, vbuf, transform, camera, extra_params);
}

// Note: this always renders a line with thickness = 1px
pub fn render_line(window: &mut Render_Window_Handle, start: &Vertex, end: &Vertex) {
    trace!("render_line");
    backend::render_line(window, start, end);
}

///////////////////////////////// QUERYING ///////////////////////////////////
pub fn get_texture_size(texture: &Texture) -> (u32, u32) {
    backend::get_texture_size(texture)
}

pub fn copy_texture_to_image(texture: &Texture) -> Image {
    backend::copy_texture_to_image(texture)
}

pub fn get_image_pixel(image: &Image, x: u32, y: u32) -> Color {
    backend::get_image_pixel(image, x, y)
}

pub fn get_image_size(image: &Image) -> (u32, u32) {
    backend::get_image_size(image)
}

pub fn get_image_pixels(image: &Image) -> &[Color] {
    backend::get_image_pixels(image)
}

pub fn get_text_size(text: &Text<'_>) -> Vec2f {
    backend::get_text_size(text)
}

pub fn shaders_are_available() -> bool {
    backend::shaders_are_available()
}

pub fn geom_shaders_are_available() -> bool {
    backend::geom_shaders_are_available()
}

pub fn vbuf_cur_vertices(vbuf: &Vertex_Buffer) -> u32 {
    backend::vbuf_cur_vertices(vbuf)
}

pub fn vbuf_max_vertices(vbuf: &Vertex_Buffer) -> u32 {
    backend::vbuf_max_vertices(vbuf)
}

///////////////////////////////// CREATING ///////////////////////////////////

pub fn create_text<'a>(string: &str, font: &'a Font<'a>, font_size: u16) -> Text<'a> {
    trace!("create_text");
    backend::create_text(string, font, font_size)
}

macro_rules! simple_wrap {
    ($newtype: ident, $wrapped: ty) => {
        pub struct $newtype($wrapped);

        impl std::ops::Deref for $newtype {
            type Target = $wrapped;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $newtype {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

simple_wrap!(Vertex_Buffer_Quads, Vertex_Buffer);
simple_wrap!(Vertex_Buffer_Triangles, Vertex_Buffer);
simple_wrap!(Vertex_Buffer_Linestrip, Vertex_Buffer);
simple_wrap!(Vertex_Buffer_Lines, Vertex_Buffer);
simple_wrap!(Vertex_Buffer_Points, Vertex_Buffer);

pub fn new_vbuf(primitive: Primitive_Type, n_vertices: u32) -> Vertex_Buffer {
    backend::new_vbuf(primitive, n_vertices)
}

pub fn vbuf_primitive_type(vbuf: &Vertex_Buffer) -> Primitive_Type {
    backend::vbuf_primitive_type(vbuf)
}
pub fn start_draw_quads(n_quads: u32) -> Vertex_Buffer_Quads {
    trace!("start_draw_quads");
    Vertex_Buffer_Quads(backend::start_draw_quads(n_quads))
}

pub fn start_draw_triangles(n_triangles: u32) -> Vertex_Buffer_Triangles {
    trace!("start_draw_triangles");
    Vertex_Buffer_Triangles(backend::start_draw_triangles(n_triangles))
}

pub fn start_draw_linestrip(n_vertices: u32) -> Vertex_Buffer_Linestrip {
    trace!("start_draw_linestrip");
    Vertex_Buffer_Linestrip(backend::start_draw_linestrip(n_vertices))
}

pub fn start_draw_lines(n_vertices: u32) -> Vertex_Buffer_Lines {
    trace!("start_draw_lines");
    Vertex_Buffer_Lines(backend::start_draw_lines(n_vertices))
}

pub fn start_draw_points(n_vertices: u32) -> Vertex_Buffer_Points {
    trace!("start_draw_points");
    Vertex_Buffer_Points(backend::start_draw_points(n_vertices))
}

///////////////////////////////// UPDATING ///////////////////////////////////

pub fn add_quad(
    vbuf: &mut Vertex_Buffer_Quads,
    v1: &Vertex,
    v2: &Vertex,
    v3: &Vertex,
    v4: &Vertex,
) {
    backend::add_quad(vbuf, v1, v2, v3, v4);
}

pub fn add_triangle(vbuf: &mut Vertex_Buffer_Triangles, v1: &Vertex, v2: &Vertex, v3: &Vertex) {
    backend::add_triangle(vbuf, v1, v2, v3);
}

pub fn add_line(vbuf: &mut Vertex_Buffer_Lines, from: &Vertex, to: &Vertex) {
    backend::add_line(vbuf, from, to);
}

pub fn add_vertex(vbuf: &mut Vertex_Buffer_Linestrip, v: &Vertex) {
    backend::add_vertex(vbuf, v);
}

pub fn add_point(vbuf: &mut Vertex_Buffer_Points, v: &Vertex) {
    backend::add_vertex(vbuf, v);
}

pub fn new_vertex(pos: Vec2f, col: Color, tex_coords: Vec2f) -> Vertex {
    backend::new_vertex(pos, col, tex_coords)
}

pub fn set_vbuf_cur_vertices(vbuf: &mut Vertex_Buffer, cur_vertices: u32) {
    backend::set_vbuf_cur_vertices(vbuf, cur_vertices);
}

pub fn swap_vbuf(a: &mut Vertex_Buffer, b: &mut Vertex_Buffer) -> bool {
    backend::swap_vbuf(a, b)
}

pub fn update_vbuf(vbuf: &mut Vertex_Buffer, vertices: &[Vertex], offset: u32) {
    backend::update_vbuf(vbuf, vertices, offset);
}

pub fn set_image_pixel(image: &mut Image, x: u32, y: u32, val: Color) {
    trace!("set_image_pixels");
    backend::set_image_pixel(image, x, y, val);
}

pub fn update_texture_pixels(texture: &mut Texture, rect: &Rect<u32>, pixels: &[Color]) {
    trace!("update_texture_pixels");
    backend::update_texture_pixels(texture, rect, pixels);
}

pub fn set_uniform_float(shader: &mut Shader, name: &str, val: f32) {
    backend::set_uniform_float(shader, name, val);
}

pub fn set_uniform_vec2(shader: &mut Shader, name: &str, val: Vec2f) {
    backend::set_uniform_vec2(shader, name, val);
}

pub fn set_uniform_color(shader: &mut Shader, name: &str, val: Color) {
    backend::set_uniform_color(shader, name, val);
}

pub fn set_uniform_texture(shader: &mut Shader, name: &str, val: &Texture) {
    backend::set_uniform_texture(shader, name, val);
}

pub fn set_texture_repeated(texture: &mut Texture, repeated: bool) {
    backend::set_texture_repeated(texture, repeated);
}
