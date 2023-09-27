use super::{Primitive_Type, Uniform_Value};
use crate::render::{Color_Type, Font_Metadata};
use crate::render_window::Render_Window_Handle;
use inle_common::colors::{self, Color, Color3};
use inle_common::paint_props::Paint_Properties;
use inle_math::matrix::Matrix3;
use inle_math::rect::Rect;
use inle_math::shapes;
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use std::ffi::CStr;
use std::marker::PhantomData;

pub struct Vertex_Buffer {
    cur_vertices: u32,
    max_vertices: u32,
    primitive_type: Primitive_Type,
}

#[derive(Copy, Clone)]
pub struct Vertex;
impl Vertex {
    pub fn color(&self) -> Color {
        colors::WHITE
    }

    pub fn set_color(&mut self, _c: Color) {}

    pub fn position(&self) -> Vec2f {
        Vec2f::default()
    }

    pub fn set_position(&mut self, _v: Vec2f) {}

    pub fn tex_coords(&self) -> Vec2f {
        Vec2f::default()
    }

    pub fn set_tex_coords(&mut self, _tc: Vec2f) {}
}

pub struct Image;
pub struct Shader;
pub struct Texture;
pub struct Font;
pub struct Text;
pub struct Uniform_Buffer;

// @Cleanup
// we should actually have API functions for these, not methods...
impl Texture {
    pub fn from_file(_: &str) -> Option<Self> {
        Some(Self)
    }
}

impl Font {
    pub fn from_file(_: &str) -> Option<Self> {
        Some(Self)
    }
}

impl Shader {
    pub fn from_memory(_: Option<&str>, _: Option<&str>, _: Option<&str>) -> Option<Self> {
        Some(Self)
    }

    pub fn from_file(_: Option<&str>, _: Option<&str>, _: Option<&str>) -> Option<Self> {
        Some(Self)
    }
}

impl Uniform_Value for f32 {
    fn apply_to(self, _shader: &mut Shader, _name: &CStr) {}
}
impl Uniform_Value for Vec2f {
    fn apply_to(self, _shader: &mut Shader, _name: &CStr) {}
}
impl Uniform_Value for &Matrix3<f32> {
    fn apply_to(self, _shader: &mut Shader, _name: &CStr) {}
}
impl Uniform_Value for Color {
    fn apply_to(self, _shader: &mut Shader, _name: &CStr) {}
}
impl Uniform_Value for Color3 {
    fn apply_to(self, _shader: &mut Shader, _name: &CStr) {}
}
impl Uniform_Value for &Texture {
    fn apply_to(self, _shader: &mut Shader, _name: &CStr) {}
}

pub fn fill_color_rect<R>(
    _window: &mut Render_Window_Handle,
    _paint_props: &Paint_Properties,
    _rect: R,
) where
    R: Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
{
}

pub fn fill_color_rect_ws<T>(
    _window: &mut Render_Window_Handle,
    _paint_props: &Paint_Properties,
    _rect: T,
    _transform: &Transform2D,
    _camera: &Transform2D,
) where
    T: std::convert::Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
{
}

pub fn fill_color_circle(
    _window: &mut Render_Window_Handle,
    _paint_props: &Paint_Properties,
    _circle: shapes::Circle,
) {
}

pub fn fill_color_circle_ws(
    _window: &mut Render_Window_Handle,
    _paint_props: &Paint_Properties,
    _circle: shapes::Circle,
    _camera: &Transform2D,
) {
}

pub fn render_text(
    _window: &mut Render_Window_Handle,
    _text: &Text,
    _paint_props: &Paint_Properties,
    _screen_pos: Vec2f,
) {
}

pub fn render_text_ws(
    _window: &mut Render_Window_Handle,
    _text: &Text,
    _paint_props: &Paint_Properties,
    _transform: &Transform2D,
    _camera: &Transform2D,
) {
}

pub fn get_texture_size(_texture: &Texture) -> (u32, u32) {
    (1, 1)
}

pub fn get_image_size(_image: &Image) -> (u32, u32) {
    (1, 1)
}

pub fn get_text_size(_text: &Text) -> Vec2f {
    Vec2f::default()
}

pub fn get_text_string(text: &Text) -> &str {
    ""
}

pub fn vbuf_primitive_type(vbuf: &Vertex_Buffer) -> Primitive_Type {
    vbuf.primitive_type
}

pub fn new_image(_width: u32, _height: u32, _color_type: Color_Type) -> Image {
    Image
}

pub fn new_image_with_data(
    _width: u32,
    _height: u32,
    _color_type: Color_Type,
    _bit_depth: u8,
    _bytes: Vec<u8>,
) -> Image {
    Image
}

pub fn new_vbuf(
    _window: &mut Render_Window_Handle,
    primitive: Primitive_Type,
    n_vertices: u32,
) -> Vertex_Buffer {
    Vertex_Buffer {
        cur_vertices: 0,
        max_vertices: n_vertices,
        primitive_type: primitive,
    }
}

pub fn new_vbuf_temp(
    _window: &mut Render_Window_Handle,
    primitive: Primitive_Type,
    n_vertices: u32,
) -> Vertex_Buffer {
    Vertex_Buffer {
        cur_vertices: 0,
        max_vertices: n_vertices,
        primitive_type: primitive,
    }
}

pub fn start_draw_quads(n_quads: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        cur_vertices: 0,
        max_vertices: n_quads * 6,
        primitive_type: Primitive_Type::Triangles,
    }
}

pub fn start_draw_triangles(n_tris: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        cur_vertices: 0,
        max_vertices: n_tris * 3,
        primitive_type: Primitive_Type::Triangles,
    }
}

pub fn start_draw_lines(n_lines: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        cur_vertices: 0,
        max_vertices: n_lines * 2,
        primitive_type: Primitive_Type::Lines,
    }
}

pub fn start_draw_linestrip(n_vertices: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        cur_vertices: 0,
        max_vertices: n_vertices,
        primitive_type: Primitive_Type::Line_Strip,
    }
}

pub fn start_draw_points(n_vertices: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        cur_vertices: 0,
        max_vertices: n_vertices,
        primitive_type: Primitive_Type::Points,
    }
}

pub fn add_quad(vbuf: &mut Vertex_Buffer, _v1: &Vertex, _v2: &Vertex, _v3: &Vertex, _v4: &Vertex) {
    vbuf.cur_vertices += 4;
}

pub fn add_triangle(vbuf: &mut Vertex_Buffer, _v1: &Vertex, _v2: &Vertex, _v3: &Vertex) {
    vbuf.cur_vertices += 3;
}

pub fn add_line(vbuf: &mut Vertex_Buffer, _from: &Vertex, _to: &Vertex) {
    vbuf.cur_vertices += 2;
}

pub fn add_vertices(vbuf: &mut Vertex_Buffer, vertices: &[Vertex]) {
    vbuf.cur_vertices += vertices.len() as u32;
}

pub fn update_vbuf(_vbuf: &mut Vertex_Buffer, _vertices: &[Vertex], _offset: u32) {}

pub fn dealloc_vbuf(_vbuf: &mut Vertex_Buffer) {}

pub fn vbuf_cur_vertices(vbuf: &Vertex_Buffer) -> u32 {
    vbuf.cur_vertices
}

pub fn vbuf_max_vertices(vbuf: &Vertex_Buffer) -> u32 {
    vbuf.max_vertices
}

pub fn set_vbuf_cur_vertices(vbuf: &mut Vertex_Buffer, cur_vertices: u32) {
    vbuf.cur_vertices = cur_vertices;
}

pub fn copy_vbuf_to_vbuf(dest: &mut Vertex_Buffer, src: &Vertex_Buffer) -> bool {
    dest.cur_vertices = src.cur_vertices;
    true
}

pub fn use_shader(_shader: &mut Shader) {}

pub fn bind_uniform_buffer(_ubo: &Uniform_Buffer) {}

pub fn uniform_buffer_needs_transfer_to_gpu(_ubo: &Uniform_Buffer) -> bool {
    false
}

pub fn create_or_get_uniform_buffer<'window>(
    _window: &'window mut Render_Window_Handle,
    _shader: &Shader,
    _name: &'static CStr,
) -> &'window mut Uniform_Buffer {
    static mut UB: Uniform_Buffer = Uniform_Buffer;
    unsafe { &mut UB }
}

pub unsafe fn write_into_uniform_buffer(
    _ubo: &mut Uniform_Buffer,
    _offset: usize,
    _align: usize,
    _size: usize,
    _data: *const u8,
) -> usize {
    0
}

pub fn new_vertex(pos: Vec2f, col: Color, tex_coords: Vec2f) -> Vertex {
    Vertex
}

pub fn render_vbuf(
    _window: &mut Render_Window_Handle,
    _vbuf: &Vertex_Buffer,
    _transform: &Transform2D,
) {
}

pub fn render_vbuf_ws(
    _window: &mut Render_Window_Handle,
    _vbuf: &Vertex_Buffer,
    _transform: &Transform2D,
    _camera: &Transform2D,
) {
}

pub fn render_vbuf_texture(
    _window: &mut Render_Window_Handle,
    _vbuf: &Vertex_Buffer,
    _texture: &Texture,
) {
}

pub fn render_vbuf_ws_with_texture(
    _window: &mut Render_Window_Handle,
    _vbuf: &Vertex_Buffer,
    _transform: &Transform2D,
    _camera: &Transform2D,
    _texture: &Texture,
) {
}

pub fn render_vbuf_with_shader(
    _window: &mut Render_Window_Handle,
    _vbuf: &Vertex_Buffer,
    _shader: &Shader,
) {
}

pub fn new_font(_atlas: Texture, _metadata: Font_Metadata) -> Font {
    Font
}

pub fn create_text(
    window: &mut Render_Window_Handle,
    _string: &str,
    _font: &Font,
    _size: u16,
) -> Text {
    Text
}

pub fn render_line(_window: &mut Render_Window_Handle, _start: &Vertex, _end: &Vertex) {}

pub fn copy_texture_to_image(_texture: &Texture) -> Image {
    Image
}

pub fn new_texture_from_image(_image: &Image, _rect: Option<Rect<i32>>) -> Texture {
    Texture
}

pub fn get_image_pixel(_image: &Image, _x: u32, _y: u32) -> Color {
    colors::TRANSPARENT
}

pub fn set_image_pixel(_image: &mut Image, _x: u32, _y: u32, _val: Color) {}

pub fn get_image_pixels(_image: &Image) -> &[Color] {
    &[]
}

pub fn swap_vbuf(_a: &mut Vertex_Buffer, _b: &mut Vertex_Buffer) -> bool {
    true
}

pub fn update_texture_pixels(_texture: &mut Texture, _rect: &Rect<u32>, _pixels: &[Color]) {}

pub fn new_shader(_vert_src: &[u8], _frag_src: &[u8], _shader_name: Option<&str>) -> Shader {
    Shader
}

pub fn shaders_are_available() -> bool {
    false
}

pub fn geom_shaders_are_available() -> bool {
    false
}

pub fn set_uniform_float(_shader: &mut Shader, _name: &str, _val: f32) {}

pub fn set_uniform_vec2(_shader: &mut Shader, _name: &str, _val: Vec2f) {}

pub fn set_uniform_color(_shader: &mut Shader, _name: &str, _val: Color) {}

pub fn set_uniform_texture(_shader: &mut Shader, _name: &str, _val: &Texture) {}

pub fn set_texture_repeated(_texture: &mut Texture, _repeated: bool) {}
