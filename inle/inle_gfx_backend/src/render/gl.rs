use super::{Primitive_Type, Render_Extra_Params};
use crate::render_window::Render_Window_Handle;
use gl::types::*;
use inle_common::colors::{self, Color};
use inle_common::paint_props::Paint_Properties;
use inle_math::rect::Rect;
use inle_math::shapes;
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use std::ffi::{c_void, CStr, CString};
use std::marker::PhantomData;
use std::mem;
use std::ptr;
use std::str;

mod shape;

pub struct Vertex_Buffer {
    cur_vertices: u32,
    max_vertices: u32,
    primitive_type: Primitive_Type,
}

#[derive(Default, Copy, Clone)]
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
    pub fn from_memory(_: Option<&str>, _: Option<&str>, _: Option<&str>) -> Option<Self> {
        Some(Self { _pd: PhantomData })
    }

    pub fn from_file(_: Option<&str>, _: Option<&str>, _: Option<&str>) -> Option<Self> {
        Some(Self { _pd: PhantomData })
    }
}

pub fn fill_color_rect<R>(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    rect: R,
) where
    R: Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
{
    let rect = rect.into();

    // @Speed: may we do this more efficiently? Maybe a single draw call?
    if paint_props.border_thick > 0. {
        let outline_rect = Rect::new(
            rect.x - paint_props.border_thick,
            rect.y - paint_props.border_thick,
            rect.width + 2. * paint_props.border_thick,
            rect.height + 2. * paint_props.border_thick,
        );
        use_rect_shader(window, paint_props.border_color, &outline_rect);
        unsafe {
            gl::BindVertexArray(window.gl.rect_vao);
            gl::DrawElements(
                gl::TRIANGLES,
                window.gl.n_rect_indices(),
                window.gl.rect_indices_type(),
                ptr::null(),
            );
        }
    }

    use_rect_shader(window, paint_props.color, &rect);

    unsafe {
        gl::BindVertexArray(window.gl.rect_vao);
        gl::DrawElements(
            gl::TRIANGLES,
            window.gl.n_rect_indices(),
            window.gl.rect_indices_type(),
            ptr::null(),
        );
    }
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

pub fn vbuf_primitive_type(vbuf: &Vertex_Buffer) -> Primitive_Type {
    vbuf.primitive_type
}

pub fn new_image(_width: u32, _height: u32) -> Image {}

pub fn new_vbuf(primitive: Primitive_Type, n_vertices: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        cur_vertices: 0,
        max_vertices: n_vertices,
        primitive_type: primitive,
    }
}

pub fn start_draw_quads(n_quads: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        cur_vertices: 0,
        max_vertices: n_quads * 4,
        primitive_type: Primitive_Type::Quads,
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

pub fn new_texture_from_image(_image: &Image, _rect: Option<Rect<i32>>) -> Option<Texture> {
    Some(Texture { _pd: PhantomData })
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

// -----------------------------------------------------------------------

macro_rules! c_str {
    ($literal:expr) => {
        CStr::from_bytes_with_nul_unchecked(concat!($literal, "\0").as_bytes())
    };
}

fn use_rect_shader(window: &mut Render_Window_Handle, color: Color, rect: &Rect<f32>) {
    let (ww, wh) = inle_win::window::get_window_target_size(window);
    let ww = 0.5 * ww as f32;
    let wh = 0.5 * wh as f32;

    unsafe {
        gl::UseProgram(window.gl.rect_shader);
        check_gl_err();

        gl::Uniform4f(
            get_uniform_loc(window.gl.rect_shader, c_str!("rect")),
            rect.x,
            rect.y,
            rect.width,
            rect.height,
        );
        check_gl_err();

        gl::Uniform2f(
            get_uniform_loc(window.gl.rect_shader, c_str!("win_half_size")),
            ww,
            wh,
        );
        check_gl_err();

        gl::Uniform4f(
            get_uniform_loc(window.gl.rect_shader, c_str!("color")),
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        );
        check_gl_err();
    }
}

#[inline]
fn get_uniform_loc(shader: GLuint, name: &CStr) -> GLint {
    unsafe {
        let loc = gl::GetUniformLocation(shader, name.as_ptr());
        #[cfg(debug_assertions)]
        if loc == -1 {
            lerr!(
                "Failed to get location of uniform `{:?}` in shader {}",
                name,
                shader
            );
        }
        loc
    }
}

#[cfg(debug_assertions)]
#[inline]
fn check_gl_err() {
    unsafe {
        let err = gl::GetError();
        match err {
            gl::NO_ERROR => {}
            gl::INVALID_ENUM => panic!("GL_INVALID_ENUM"),
            gl::INVALID_OPERATION => panic!("GL_INVALID_OPERATION"),
            gl::INVALID_VALUE => panic!("GL_INVALID_VALUE"),
            _ => panic!("Other GL error: {}", err),
        }
    }
}

#[cfg(not(debug_assertions))]
fn check_gl_err() {}
