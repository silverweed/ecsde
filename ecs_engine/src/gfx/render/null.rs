use super::Render_Extra_Params;
use crate::common::colors::{self, Color};
use crate::common::rect::Rect;
use crate::common::shapes;
use crate::common::transform::Transform2D;
use crate::common::vector::Vec2f;
use crate::gfx::paint_props::Paint_Properties;
use crate::gfx::render_window::Render_Window_Handle;
use std::marker::PhantomData;

pub struct Vertex_Buffer {
    cur_vertices: u32,
    max_vertices: u32,
}
#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: Vec2f,
    pub color: Color,
    pub tex_coords: Vec2f,
}
pub type Image = ();
pub struct Shader<'texture> {
    _pd: PhantomData<&'texture ()>,
}
pub struct Texture<'a> {
    _pd: PhantomData<&'a ()>,
}
pub struct Font<'a> {
    _pd: PhantomData<&'a ()>,
}
pub struct Text<'font> {
    _pd: PhantomData<&'font ()>,
}

// @Cleanup
// we should actually have API functions for these, not methods...
impl Texture<'_> {
    pub fn from_file(_: &str) -> Option<Self> {
        Some(Self { _pd: PhantomData })
    }
}

impl Font<'_> {
    pub fn from_file(_: &str) -> Option<Self> {
        Some(Self { _pd: PhantomData })
    }
}

impl Shader<'_> {
    pub fn from_file(_: Option<&str>, _: Option<&str>, _: Option<&str>) -> Option<Self> {
        Some(Self { _pd: PhantomData })
    }
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
    _text: &mut Text,
    _paint_props: &Paint_Properties,
    _screen_pos: Vec2f,
) {
}

pub fn render_text_ws(
    _window: &mut Render_Window_Handle,
    _text: &mut Text,
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

pub fn start_draw_quads(n_quads: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        cur_vertices: 0,
        max_vertices: n_quads * 4,
    }
}

pub fn start_draw_triangles(n_tris: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        cur_vertices: 0,
        max_vertices: n_tris * 3,
    }
}

pub fn start_draw_lines(n_lines: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        cur_vertices: 0,
        max_vertices: n_lines * 2,
    }
}

pub fn start_draw_linestrip(n_vertices: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        cur_vertices: 0,
        max_vertices: n_vertices,
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

pub fn add_vertex(vbuf: &mut Vertex_Buffer, _v: &Vertex) {
    vbuf.cur_vertices += 1;
}

pub fn update_vbuf(_vbuf: &mut Vertex_Buffer, _vertices: &[Vertex], _offset: u32) {}

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

pub fn new_vertex(pos: Vec2f, col: Color, tex_coords: Vec2f) -> Vertex {
    Vertex {
        position: pos,
        color: col,
        tex_coords,
    }
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

pub fn render_vbuf_ws_ex(
    _window: &mut Render_Window_Handle,
    _vbuf: &Vertex_Buffer,
    _transform: &Transform2D,
    _camera: &Transform2D,
    _extra_params: Render_Extra_Params,
) {
}

pub fn render_vbuf_texture(
    _window: &mut Render_Window_Handle,
    _vbuf: &Vertex_Buffer,
    _texture: &Texture,
) {
}

pub fn create_text<'a>(_string: &str, _font: &'a Font, _size: u16) -> Text<'a> {
    Text { _pd: PhantomData }
}

pub fn render_line(_window: &mut Render_Window_Handle, _start: &Vertex, _end: &Vertex) {}

pub fn copy_texture_to_image(_texture: &Texture) -> Image {}

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

pub fn shaders_are_available() -> bool {
    false
}

pub fn set_uniform_float(_shader: &mut Shader, _name: &str, _val: f32) {}

pub fn set_uniform_vec2(_shader: &mut Shader, _name: &str, _val: Vec2f) {}

pub fn set_uniform_color(_shader: &mut Shader, _name: &str, _val: Color) {}

pub fn set_uniform_texture(_shader: &mut Shader, _name: &str, _val: &Texture) {}

pub fn set_texture_repeated(_texture: &mut Texture, _repeated: bool) {}
