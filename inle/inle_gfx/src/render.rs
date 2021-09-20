use crate::material::Material;
use inle_common::colors::Color;
use inle_common::paint_props::Paint_Properties;
use inle_gfx_backend::render::backend;
use inle_gfx_backend::render_window::Render_Window_Handle;
use inle_math::rect::Rect;
use inle_math::shapes::Circle;
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;

pub use inle_gfx_backend::render::{Primitive_Type, Uniform_Value};

pub mod batcher;

pub type Z_Index = i8;
pub type Font<'a> = backend::Font<'a>;
pub type Image = backend::Image;
pub type Shader<'a> = backend::Shader<'a>;
pub type Text<'a> = backend::Text<'a>;
pub type Texture<'a> = backend::Texture<'a>;
pub type Vertex_Buffer = backend::Vertex_Buffer;
pub type Vertex = backend::Vertex;
pub type Color_Type = backend::Color_Type;
pub type Uniform_Buffer = backend::Uniform_Buffer;

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
    window: &mut Render_Window_Handle,
    batches: &mut batcher::Batches,
    material: &Material,
    tex_rect: &Rect<i32>,
    color: Color,
    transform: &Transform2D,
    z_index: Z_Index,
) {
    trace!("render_texture_ws");
    batcher::add_texture_ws(
        window, batches, material, tex_rect, color, transform, z_index,
    );
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

pub fn render_vbuf_ws_with_texture(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
    camera: &Transform2D,
    texture: &Texture,
) {
    trace!("render_vbuf_ws_with_texture");
    backend::render_vbuf_ws_with_texture(window, vbuf, transform, camera, texture);
}

pub fn render_vbuf_with_shader(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    shader: &Shader,
) {
    trace!("render_vbuf_with_shader");
    backend::render_vbuf_with_shader(window, vbuf, shader);
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

pub fn new_texture_from_image(image: &Image, rect: Option<Rect<i32>>) -> Texture {
    backend::new_texture_from_image(image, rect)
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

pub fn get_text_string<'a>(text: &'a Text) -> &'a str {
    backend::get_text_string(text)
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

pub fn new_image(width: u32, height: u32, color_type: Color_Type) -> Image {
    backend::new_image(width, height, color_type)
}

pub fn new_vbuf(
    window: &mut Render_Window_Handle,
    primitive: Primitive_Type,
    n_vertices: u32,
) -> Vertex_Buffer {
    trace!("new_vbuf");
    backend::new_vbuf(window, primitive, n_vertices)
}

/// Creates a Vertex_Buffer that gets deallocated automatically at the end of the frame
pub fn new_vbuf_temp(
    window: &mut Render_Window_Handle,
    primitive: Primitive_Type,
    n_vertices: u32,
) -> Vertex_Buffer {
    trace!("new_vbuf_temp");
    backend::new_vbuf_temp(window, primitive, n_vertices)
}

pub fn vbuf_primitive_type(vbuf: &Vertex_Buffer) -> Primitive_Type {
    backend::vbuf_primitive_type(vbuf)
}

pub fn start_draw_quads_temp(
    window: &mut Render_Window_Handle,
    n_quads: u32,
) -> Vertex_Buffer_Quads {
    Vertex_Buffer_Quads(new_vbuf_temp(
        window,
        Primitive_Type::Triangles,
        n_quads * 6,
    ))
}

pub fn start_draw_triangles_temp(
    window: &mut Render_Window_Handle,
    n_triangles: u32,
) -> Vertex_Buffer_Triangles {
    Vertex_Buffer_Triangles(new_vbuf_temp(
        window,
        Primitive_Type::Triangles,
        n_triangles * 3,
    ))
}

pub fn start_draw_linestrip_temp(
    window: &mut Render_Window_Handle,
    n_vertices: u32,
) -> Vertex_Buffer_Linestrip {
    Vertex_Buffer_Linestrip(new_vbuf_temp(
        window,
        Primitive_Type::Line_Strip,
        n_vertices,
    ))
}

pub fn start_draw_lines_temp(
    window: &mut Render_Window_Handle,
    n_lines: u32,
) -> Vertex_Buffer_Lines {
    Vertex_Buffer_Lines(new_vbuf_temp(window, Primitive_Type::Lines, n_lines * 2))
}

pub fn start_draw_points_temp(
    window: &mut Render_Window_Handle,
    n_vertices: u32,
) -> Vertex_Buffer_Points {
    Vertex_Buffer_Points(new_vbuf_temp(window, Primitive_Type::Points, n_vertices))
}

///////////////////////////////// UPDATING ///////////////////////////////////

#[inline]
pub fn add_quad(
    vbuf: &mut Vertex_Buffer_Quads,
    v1: &Vertex,
    v2: &Vertex,
    v3: &Vertex,
    v4: &Vertex,
) {
    backend::add_vertices(vbuf, &[*v1, *v2, *v3, *v3, *v4, *v1]);
}

#[inline]
pub fn add_triangle(vbuf: &mut Vertex_Buffer_Triangles, v1: &Vertex, v2: &Vertex, v3: &Vertex) {
    backend::add_vertices(vbuf, &[*v1, *v2, *v3]);
}

#[inline]
pub fn add_line(vbuf: &mut Vertex_Buffer_Lines, from: &Vertex, to: &Vertex) {
    backend::add_vertices(vbuf, &[*from, *to]);
}

#[inline]
pub fn add_vertex(vbuf: &mut Vertex_Buffer_Linestrip, v: &Vertex) {
    backend::add_vertices(vbuf, &[*v]);
}

#[inline]
pub fn add_point(vbuf: &mut Vertex_Buffer_Points, v: &Vertex) {
    backend::add_vertices(vbuf, &[*v]);
}

#[inline]
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

pub fn use_shader(shader: &mut Shader) {
    backend::use_shader(shader);
}

pub fn set_uniform<T: Uniform_Value>(shader: &mut Shader, name: &std::ffi::CStr, val: T) {
    inle_gfx_backend::render::set_uniform(shader, name, val);
}

pub fn set_texture_repeated(texture: &mut Texture, repeated: bool) {
    backend::set_texture_repeated(texture, repeated);
}

pub fn set_texture_smooth(texture: &mut Texture, smooth: bool) {
    backend::set_texture_repeated(texture, smooth);
}

pub fn create_or_get_uniform_buffer<'window>(
    window: &'window mut Render_Window_Handle,
    shader: &Shader,
    name: &'static std::ffi::CStr,
) -> &'window mut Uniform_Buffer {
    backend::create_or_get_uniform_buffer(window, shader, name)
}

/// # Safety
/// The struct must respect these specs for the std140 layout:
/// https://www.khronos.org/registry/OpenGL/specs/gl/glspec45.core.pdf#page=159
pub unsafe trait Std140 {}

/// Returns the offset where to write next
pub fn write_into_uniform_buffer<T: Std140>(
    ubo: &mut Uniform_Buffer,
    offset: usize,
    data: T,
) -> usize {
    let align = std::mem::align_of::<T>();
    let size = std::mem::size_of::<T>();
    unsafe {
        backend::write_into_uniform_buffer(ubo, offset, align, size, &data as *const _ as *const u8)
    }
}

/// Returns the offset where to write next
pub fn write_array_into_uniform_buffer<T: Std140>(
    ubo: &mut Uniform_Buffer,
    offset: usize,
    data: &[T],
) -> usize {
    let align = std::mem::align_of::<T>();
    let size = std::mem::size_of::<T>();
    unsafe {
        backend::write_into_uniform_buffer(
            ubo,
            offset,
            align,
            size * data.len(),
            data.as_ptr() as *const u8,
        )
    }
}

pub fn bind_uniform_buffer(ubo: &Uniform_Buffer) {
    backend::bind_uniform_buffer(ubo);
}

#[inline]
pub fn uniform_buffer_needs_transfer_to_gpu(ubo: &Uniform_Buffer) -> bool {
    backend::uniform_buffer_needs_transfer_to_gpu(ubo)
}
