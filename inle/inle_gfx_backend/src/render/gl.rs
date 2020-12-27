use super::{Primitive_Type, Render_Extra_Params};
use crate::render_window::Render_Window_Handle;
use gl::types::*;
use inle_common::colors::{self, Color};
use inle_common::paint_props::Paint_Properties;
use inle_math::rect::Rect;
use inle_math::shapes;
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use std::ffi::{c_void, CString};
use std::marker::PhantomData;
use std::mem;
use std::ptr;
use std::str;

pub struct Vertex_Buffer {
    cur_vertices: u32,
    max_vertices: u32,
    primitive_type: Primitive_Type,
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
    // @Temporary crap code just to see if we can get something on screen.
    // Apparently we can!

    let mut rect = rect.into();
    let (ww, wh) = inle_win::window::get_window_target_size(window);
    let ww = 0.5 * ww as f32;
    let wh = 0.5 * wh as f32;
    rect.x = (rect.x - ww) / ww;
    rect.y = -(rect.y - wh) / wh;
    rect.width /= ww;
    rect.height /= -wh;

    let vertexShaderSource = "
            #version 330 core
            layout (location = 0) in vec2 aPos;
            void main() {
               gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0);
            }
    ";
    let fragmentShaderSource = "
            #version 330 core
            uniform vec3 color;
            out vec4 FragColor;
            void main() {
               FragColor = vec4(color, 1.0);
            }
    ";

    unsafe {
        let vertexShader = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_vert = CString::new(vertexShaderSource.as_bytes()).unwrap();
        gl::ShaderSource(vertexShader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(vertexShader);

        let mut success = gl::FALSE as GLint;
        let mut infoLog = Vec::with_capacity(512);
        infoLog.set_len(512 - 1); // subtract 1 to skip the trailing null character
        gl::GetShaderiv(vertexShader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(
                vertexShader,
                512,
                ptr::null_mut(),
                infoLog.as_mut_ptr() as *mut GLchar,
            );
            println!(
                "ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}",
                str::from_utf8(&infoLog).unwrap()
            );
        }

        // fragment shader
        let fragmentShader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_frag = CString::new(fragmentShaderSource.as_bytes()).unwrap();
        gl::ShaderSource(fragmentShader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragmentShader);
        // check for shader compile errors
        gl::GetShaderiv(fragmentShader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(
                fragmentShader,
                512,
                ptr::null_mut(),
                infoLog.as_mut_ptr() as *mut GLchar,
            );
            println!(
                "ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}",
                str::from_utf8(&infoLog).unwrap()
            );
        }

        // link shaders
        let shaderProgram = gl::CreateProgram();
        gl::AttachShader(shaderProgram, vertexShader);
        gl::AttachShader(shaderProgram, fragmentShader);
        gl::LinkProgram(shaderProgram);
        // check for linking errors
        gl::GetProgramiv(shaderProgram, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetProgramInfoLog(
                shaderProgram,
                512,
                ptr::null_mut(),
                infoLog.as_mut_ptr() as *mut GLchar,
            );
            println!(
                "ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}",
                str::from_utf8(&infoLog).unwrap()
            );
        }
        gl::DeleteShader(vertexShader);
        gl::DeleteShader(fragmentShader);

        //let vertices: [f32; 9] = [
        //-0.5, -0.5, 0.0, // left
        //0.5, -0.5, 0.0, // right
        //0.0, 0.5, 0.0, // top
        //];
        let vertices = [
            rect.x,
            rect.y,
            rect.x + rect.width,
            rect.y,
            rect.x + rect.width,
            rect.y + rect.height,
            rect.x + rect.width,
            rect.y + rect.height,
            rect.x,
            rect.y + rect.height,
            rect.x,
            rect.y,
        ];
        let (mut VBO, mut VAO) = (0, 0);
        gl::GenVertexArrays(1, &mut VAO);
        gl::GenBuffers(1, &mut VBO);
        // bind the Vertex Array Object first, then bind and set vertex buffer(s), and then configure vertex attributes(s).
        gl::BindVertexArray(VAO);

        gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &vertices[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            2 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        // note that this is allowed, the call to gl::VertexAttribPointer registered VBO as the vertex attribute's bound vertex buffer object so afterwards we can safely unbind
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        // You can unbind the VAO afterwards so other VAO calls won't accidentally modify this VAO, but this rarely happens. Modifying other
        // VAOs requires a call to glBindVertexArray anyways so we generally don't unbind VAOs (nor VBOs) when it's not directly necessary.
        gl::BindVertexArray(0);

        //gl::ClearColor(0.2, 0.3, 0.3, 1.0);
        //gl::Clear(gl::COLOR_BUFFER_BIT);

        // draw our first triangle
        gl::UseProgram(shaderProgram);

        gl::Uniform3f(
            gl::GetUniformLocation(shaderProgram, "color".as_ptr() as _),
            paint_props.color.r as f32 / 255.0,
            paint_props.color.g as f32 / 255.0,
            paint_props.color.b as f32 / 255.0,
        );

        gl::BindVertexArray(VAO); // seeing as we only have a single VAO there's no need to bind it every time, but we'll do so to keep things a bit more organized
        gl::DrawArrays(gl::TRIANGLES, 0, vertices.len() as i32 / 2);
        gl::BindVertexArray(0); // no need to unbind it every time
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

pub fn new_image(_width: u32, _height: u32) -> Image {
    ()
}

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
