use super::{Primitive_Type, Render_Extra_Params};
use crate::render_window::Render_Window_Handle;
use gl::types::*;
use inle_common::colors::{self, Color};
use inle_common::paint_props::Paint_Properties;
use inle_math::matrix::Matrix3;
use inle_math::rect::Rect;
use inle_math::shapes;
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use std::ffi::{c_void, CStr, CString};
use std::marker::PhantomData;
use std::{mem, ptr, str};

fn to_gl_primitive_type(prim: Primitive_Type) -> GLenum {
    match prim {
        Primitive_Type::Points => gl::POINTS,
        Primitive_Type::Lines => gl::LINES,
        Primitive_Type::Line_Strip => gl::LINE_STRIP,
        Primitive_Type::Triangles => gl::TRIANGLES,
        Primitive_Type::Triangle_Strip => gl::TRIANGLE_STRIP,
        Primitive_Type::Triangle_Fan => gl::TRIANGLE_FAN,
        Primitive_Type::Quads => gl::QUADS,
    }
}

pub struct Vertex_Buffer {
    cur_vertices: u32,
    max_vertices: u32,
    primitive_type: Primitive_Type,
    id: GLuint,
}

impl Vertex_Buffer {
    const LOC_IN_POS: GLuint = 0;
    const LOC_IN_COLOR: GLuint = 1;
    const LOC_IN_TEXCOORD: GLuint = 2;

    fn new(primitive_type: Primitive_Type, max_vertices: u32) -> Self {
        let mut id = 0;
        if max_vertices > 0 {
            unsafe {
                gl::GenBuffers(1, &mut id);

                gl::BindBuffer(gl::ARRAY_BUFFER, id);
                check_gl_err();

                // NOTE: using a BufferStorage means that this buffer is unresizable.
                gl::BufferStorage(
                    gl::ARRAY_BUFFER,
                    (max_vertices * mem::size_of::<Vertex>() as u32) as _,
                    ptr::null(),
                    gl::DYNAMIC_STORAGE_BIT,
                );
                check_gl_err();

                gl::VertexAttribPointer(
                    Self::LOC_IN_POS,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    2 * mem::size_of::<GLfloat>() as GLsizei,
                    ptr::null(),
                );
                gl::EnableVertexAttribArray(Self::LOC_IN_POS);

                gl::VertexAttribPointer(
                    Self::LOC_IN_COLOR,
                    4,
                    gl::FLOAT,
                    gl::FALSE,
                    4 * mem::size_of::<GLfloat>() as GLsizei,
                    ptr::null(),
                );
                gl::EnableVertexAttribArray(Self::LOC_IN_COLOR);

                gl::VertexAttribPointer(
                    Self::LOC_IN_TEXCOORD,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    2 * mem::size_of::<GLfloat>() as GLsizei,
                    ptr::null(),
                );
                gl::EnableVertexAttribArray(Self::LOC_IN_TEXCOORD);
            }
        } else {
            lwarn!("Creating a Vertex_Buffer with max_vertices = 0");
        }

        Self {
            id,
            cur_vertices: 0,
            max_vertices,
            primitive_type,
        }
    }

    fn add_vertices(&mut self, vertices: &[Vertex]) {
        debug_assert!(self.cur_vertices as usize + vertices.len() <= self.max_vertices as usize);
        unsafe {
            gl::NamedBufferSubData(
                self.id,
                (self.cur_vertices * mem::size_of::<Vertex>() as u32) as _,
                (vertices.len() * mem::size_of::<Vertex>()) as _,
                vertices.as_ptr() as _,
            );
            check_gl_err();
        }
        debug_assert!(self.cur_vertices as usize + vertices.len() < std::u32::MAX as usize);
        self.cur_vertices += vertices.len() as u32;
    }

    fn update_vertices(&mut self, vertices: &[Vertex], offset: u32) {
        unsafe {
            gl::NamedBufferSubData(
                self.id,
                (offset as usize * mem::size_of::<Vertex>()) as _,
                (vertices.len() * mem::size_of::<Vertex>()) as _,
                vertices.as_ptr() as _,
            );
            check_gl_err();
        }
    }
}

impl Drop for Vertex_Buffer {
    fn drop(&mut self) {
        // @FIXME: we are crashing here on shutdown because the window is destroyed before this happens!
        // We should avoid RAII for these buffers, and instead use a sort of arena-allocator approach, which
        // we can then free just before dropping the game state.
        unsafe {
            gl::DeleteBuffers(1, &mut self.id);
        }
    }
}

#[repr(C)]
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
            window
                .gl
                .draw_indexed(window.gl.n_rect_indices(), window.gl.rect_indices_type());
        }
    }

    use_rect_shader(window, paint_props.color, &rect);

    unsafe {
        gl::BindVertexArray(window.gl.rect_vao);
        window
            .gl
            .draw_indexed(window.gl.n_rect_indices(), window.gl.rect_indices_type());
    }
}

pub fn fill_color_rect_ws<T>(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    rect: T,
    transform: &Transform2D,
    camera: &Transform2D,
) where
    T: std::convert::Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
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
        use_rect_ws_shader(
            window,
            paint_props.border_color,
            &outline_rect,
            transform,
            camera,
        );
        unsafe {
            gl::BindVertexArray(window.gl.rect_vao);
            window
                .gl
                .draw_indexed(window.gl.n_rect_indices(), window.gl.rect_indices_type());
        }
    }

    use_rect_ws_shader(window, paint_props.color, &rect, transform, camera);

    unsafe {
        gl::BindVertexArray(window.gl.rect_vao);
        window
            .gl
            .draw_indexed(window.gl.n_rect_indices(), window.gl.rect_indices_type());
    }
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
    Vertex_Buffer::new(primitive, n_vertices)
}

pub fn start_draw_quads(n_quads: u32) -> Vertex_Buffer {
    Vertex_Buffer::new(Primitive_Type::Quads, n_quads * 4)
}

pub fn start_draw_triangles(n_tris: u32) -> Vertex_Buffer {
    Vertex_Buffer::new(Primitive_Type::Triangles, n_tris * 4)
}

pub fn start_draw_lines(n_lines: u32) -> Vertex_Buffer {
    Vertex_Buffer::new(Primitive_Type::Lines, n_lines * 2)
}

pub fn start_draw_linestrip(n_vertices: u32) -> Vertex_Buffer {
    Vertex_Buffer::new(Primitive_Type::Line_Strip, n_vertices)
}

pub fn start_draw_points(n_vertices: u32) -> Vertex_Buffer {
    Vertex_Buffer::new(Primitive_Type::Points, n_vertices)
}

pub fn add_quad(vbuf: &mut Vertex_Buffer, v1: &Vertex, v2: &Vertex, v3: &Vertex, v4: &Vertex) {
    vbuf.add_vertices(&[*v1, *v2, *v3, *v4]);
}

pub fn add_triangle(vbuf: &mut Vertex_Buffer, v1: &Vertex, v2: &Vertex, v3: &Vertex) {
    vbuf.add_vertices(&[*v1, *v2, *v3]);
}

pub fn add_line(vbuf: &mut Vertex_Buffer, from: &Vertex, to: &Vertex) {
    vbuf.add_vertices(&[*from, *to]);
}

pub fn add_vertex(vbuf: &mut Vertex_Buffer, v: &Vertex) {
    vbuf.add_vertices(&[*v]);
}

pub fn update_vbuf(vbuf: &mut Vertex_Buffer, vertices: &[Vertex], offset: u32) {
    vbuf.update_vertices(vertices, offset);
}

pub fn vbuf_cur_vertices(vbuf: &Vertex_Buffer) -> u32 {
    vbuf.cur_vertices
}

pub fn vbuf_max_vertices(vbuf: &Vertex_Buffer) -> u32 {
    vbuf.max_vertices
}

pub fn set_vbuf_cur_vertices(vbuf: &mut Vertex_Buffer, cur_vertices: u32) {
    // TODO
    vbuf.cur_vertices = cur_vertices;
}

pub fn new_vertex(pos: Vec2f, col: Color, tex_coords: Vec2f) -> Vertex {
    Vertex {
        position: pos,
        color: col,
        tex_coords,
    }
}

pub fn render_vbuf(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
) {
    use_vbuf_shader(window, transform);

    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, vbuf.id);
        check_gl_err();
        gl::DrawArrays(
            to_gl_primitive_type(vbuf.primitive_type),
            0,
            vbuf.cur_vertices as _,
        );
    }
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

pub fn render_line(window: &mut Render_Window_Handle, start: &Vertex, end: &Vertex) {
    // TODO
    //use_line_shader();
}

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

    let vertices = {
        let mut rect = *rect;
        rect.x = (rect.x - ww) / ww;
        rect.y = (wh - rect.y) / wh;
        rect.width /= ww;
        rect.height /= -wh;

        // @Volatile: order must be consistent with render_window::backend::RECT_INDICES
        [
            rect.x,
            rect.y,
            rect.x + rect.width,
            rect.y,
            rect.x + rect.width,
            rect.y + rect.height,
            rect.x,
            rect.y + rect.height,
        ]
    };

    // @TODO: consider using UBOs
    unsafe {
        gl::UseProgram(window.gl.rect_shader);
        check_gl_err();

        gl::Uniform2fv(
            get_uniform_loc(window.gl.rect_shader, c_str!("vertices")),
            4,
            vertices.as_ptr(),
        );

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

fn use_rect_ws_shader(
    window: &mut Render_Window_Handle,
    color: Color,
    rect: &Rect<f32>,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    let (width, height) = inle_win::window::get_window_target_size(window);
    let model = transform;
    let view = camera.inverse();
    let projection = Matrix3::new(
        2. / width as f32,
        0.,
        0.,
        0.,
        -2. / height as f32,
        0.,
        0.,
        0.,
        1.,
    );
    let mvp = projection * view.get_matrix() * model.get_matrix();

    // @Volatile: order must be consistent with render_window::backend::RECT_INDICES
    let rect_vertices = [
        rect.x,
        rect.y,
        rect.x + rect.width,
        rect.y,
        rect.x + rect.width,
        rect.y + rect.height,
        rect.x,
        rect.y + rect.height,
    ];

    // @TODO: consider using UBOs
    unsafe {
        gl::UseProgram(window.gl.rect_ws_shader);
        check_gl_err();

        gl::UniformMatrix3fv(
            get_uniform_loc(window.gl.rect_ws_shader, c_str!("mvp")),
            1,
            gl::FALSE,
            mvp.as_slice().as_ptr(),
        );
        check_gl_err();

        gl::Uniform2fv(
            get_uniform_loc(window.gl.rect_ws_shader, c_str!("rect")),
            (rect_vertices.len() / 2) as _,
            rect_vertices.as_ptr(),
        );

        gl::Uniform4f(
            get_uniform_loc(window.gl.rect_ws_shader, c_str!("color")),
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        );
        check_gl_err();
    }
}

fn use_vbuf_shader(window: &mut Render_Window_Handle, transform: &Transform2D) {
    unsafe {
        gl::UseProgram(window.gl.vbuf_shader);

        gl::UniformMatrix3fv(
            get_uniform_loc(window.gl.vbuf_shader, c_str!("transform")),
            1,
            gl::FALSE,
            transform.get_matrix().as_slice().as_ptr(),
        );
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
#[track_caller]
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
