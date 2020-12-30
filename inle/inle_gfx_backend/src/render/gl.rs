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
    }
}

pub struct Vertex_Buffer {
    cur_vertices: u32,
    max_vertices: u32,
    primitive_type: Primitive_Type,
    vbo: GLuint,
    vao: GLuint,
}

impl Vertex_Buffer {
    const LOC_IN_COLOR: GLuint = 0;
    const LOC_IN_POS: GLuint = 1;
    const LOC_IN_TEXCOORD: GLuint = 2;

    fn new(primitive_type: Primitive_Type, max_vertices: u32) -> Self {
        let mut vao = 0;
        let mut vbo = 0;
        if max_vertices > 0 {
            unsafe {
                gl::GenVertexArrays(1, &mut vao);
                gl::GenBuffers(1, &mut vbo);

                debug_assert!(vao != 0);
                debug_assert!(vbo != 0);

                gl::BindVertexArray(vao);

                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
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
                    Self::LOC_IN_COLOR,
                    4,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<Vertex>() as _,
                    // @Robustness: use offsetof or similar
                    ptr::null(),
                );
                gl::EnableVertexAttribArray(Self::LOC_IN_COLOR);

                gl::VertexAttribPointer(
                    Self::LOC_IN_POS,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<Vertex>() as _,
                    // @Robustness: use offsetof or similar
                    mem::size_of::<Glsl_Vec4>() as *const c_void,
                );
                gl::EnableVertexAttribArray(Self::LOC_IN_POS);

                gl::VertexAttribPointer(
                    Self::LOC_IN_TEXCOORD,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<Vertex>() as _,
                    // @Robustness: use offsetof or similar
                    (mem::size_of::<Glsl_Vec4>() + mem::size_of::<Vec2f>()) as *const c_void,
                );
                gl::EnableVertexAttribArray(Self::LOC_IN_TEXCOORD);
            }
        } else {
            lwarn!("Creating a Vertex_Buffer with max_vertices = 0");
        }

        Self {
            vao,
            vbo,
            cur_vertices: 0,
            max_vertices,
            primitive_type,
        }
    }
}

impl Drop for Vertex_Buffer {
    fn drop(&mut self) {
        // @FIXME: we are crashing here on shutdown because the window is destroyed before this happens!
        // We should avoid RAII for these buffers, and instead use a sort of arena-allocator approach, which
        // we can then free just before dropping the game state.
        unsafe {
            gl::DeleteBuffers(1, &mut self.vbo);
            gl::DeleteVertexArrays(1, &mut self.vao);
        }
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
struct Glsl_Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

const_assert!(mem::size_of::<Glsl_Vec4>() == mem::size_of::<GLfloat>() * 4);
const_assert!(mem::size_of::<Vec2f>() == mem::size_of::<GLfloat>() * 2);

impl From<Color> for Glsl_Vec4 {
    fn from(c: Color) -> Self {
        Self {
            x: c.r as f32 / 255.0,
            y: c.g as f32 / 255.0,
            z: c.b as f32 / 255.0,
            w: c.a as f32 / 255.0,
        }
    }
}

impl From<Glsl_Vec4> for Color {
    fn from(c: Glsl_Vec4) -> Self {
        Self {
            r: (c.x * 255.0) as u8,
            g: (c.y * 255.0) as u8,
            b: (c.z * 255.0) as u8,
            a: (c.w * 255.0) as u8,
        }
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Vertex {
    color: Glsl_Vec4,  // 16 B
    position: Vec2f,   // 8 B
    tex_coords: Vec2f, // 8 B
}

impl Vertex {
    pub fn color(&self) -> Color {
        self.color.into()
    }

    pub fn set_color(&mut self, c: Color) {
        self.color = c.into();
    }

    pub fn position(&self) -> Vec2f {
        self.position
    }

    pub fn set_position(&mut self, v: Vec2f) {
        self.position = v;
    }

    pub fn tex_coords(&self) -> Vec2f {
        self.tex_coords
    }

    pub fn set_tex_coords(&mut self, tc: Vec2f) {
        self.tex_coords = tc;
    }
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
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    circle: shapes::Circle,
) {
    if paint_props.border_thick > 0. {
        let outline = shapes::Circle {
            center: circle.center,
            radius: circle.radius + 2. * paint_props.border_thick,
        };

        use_circle_shader(window, paint_props.border_color, outline);

        unsafe {
            gl::BindVertexArray(window.gl.circle_vao);
            window
                .gl
                .draw_arrays(gl::TRIANGLE_FAN, 0, window.gl.n_circle_vertices());
        }
    }

    use_circle_shader(window, paint_props.color, circle);

    unsafe {
        gl::BindVertexArray(window.gl.circle_vao);
        window
            .gl
            .draw_arrays(gl::TRIANGLE_FAN, 0, window.gl.n_circle_vertices());
    }
}

pub fn fill_color_circle_ws(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    circle: shapes::Circle,
    camera: &Transform2D,
) {
    if paint_props.border_thick > 0. {
        let outline = shapes::Circle {
            center: circle.center,
            radius: circle.radius + 2. * paint_props.border_thick,
        };

        use_circle_ws_shader(window, paint_props.border_color, outline, camera);

        unsafe {
            gl::BindVertexArray(window.gl.circle_vao);
            window
                .gl
                .draw_arrays(gl::TRIANGLE_FAN, 0, window.gl.n_circle_vertices());
        }
    }

    use_circle_ws_shader(window, paint_props.color, circle, camera);

    unsafe {
        gl::BindVertexArray(window.gl.circle_vao);
        window
            .gl
            .draw_arrays(gl::TRIANGLE_FAN, 0, window.gl.n_circle_vertices());
    }
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

pub fn new_image(_width: u32, _height: u32) -> Image {}

#[inline(always)]
pub fn vbuf_primitive_type(vbuf: &Vertex_Buffer) -> Primitive_Type {
    vbuf.primitive_type
}

#[inline(always)]
pub fn new_vbuf(primitive: Primitive_Type, n_vertices: u32) -> Vertex_Buffer {
    Vertex_Buffer::new(primitive, n_vertices)
}

#[inline(always)]
pub fn add_vertices(vbuf: &mut Vertex_Buffer, vertices: &[Vertex]) {
    debug_assert!(vbuf.cur_vertices as usize + vertices.len() <= vbuf.max_vertices as usize);
    unsafe {
        gl::NamedBufferSubData(
            vbuf.vbo,
            (vbuf.cur_vertices * mem::size_of::<Vertex>() as u32) as _,
            (vertices.len() * mem::size_of::<Vertex>()) as _,
            vertices.as_ptr() as _,
        );
        check_gl_err();
    }
    debug_assert!(vbuf.cur_vertices as usize + vertices.len() < std::u32::MAX as usize);
    vbuf.cur_vertices += vertices.len() as u32;
}

#[inline(always)]
pub fn update_vbuf(vbuf: &mut Vertex_Buffer, vertices: &[Vertex], offset: u32) {
    unsafe {
        gl::NamedBufferSubData(
            vbuf.vbo,
            (offset as usize * mem::size_of::<Vertex>()) as _,
            (vertices.len() * mem::size_of::<Vertex>()) as _,
            vertices.as_ptr() as _,
        );
        check_gl_err();
    }
}

#[inline(always)]
pub fn vbuf_cur_vertices(vbuf: &Vertex_Buffer) -> u32 {
    vbuf.cur_vertices
}

#[inline(always)]
pub fn vbuf_max_vertices(vbuf: &Vertex_Buffer) -> u32 {
    vbuf.max_vertices
}

#[inline(always)]
pub fn set_vbuf_cur_vertices(vbuf: &mut Vertex_Buffer, cur_vertices: u32) {
    vbuf.cur_vertices = cur_vertices;
}

pub fn new_vertex(pos: Vec2f, col: Color, tex_coords: Vec2f) -> Vertex {
    Vertex {
        position: pos,
        color: col.into(),
        tex_coords,
    }
}

pub fn render_vbuf(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
) {
    if vbuf_cur_vertices(vbuf) == 0 {
        return;
    }

    use_vbuf_shader(window, transform);

    unsafe {
        gl::BindVertexArray(vbuf.vao);
        check_gl_err();

        window.gl.draw_arrays(
            to_gl_primitive_type(vbuf.primitive_type),
            0,
            vbuf.cur_vertices as _,
        );
        check_gl_err();
    }
}

pub fn render_vbuf_ws(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    if vbuf_cur_vertices(vbuf) == 0 {
        return;
    }

    use_vbuf_ws_shader(window, transform, camera);

    unsafe {
        gl::BindVertexArray(vbuf.vao);
        check_gl_err();

        window.gl.draw_arrays(
            to_gl_primitive_type(vbuf.primitive_type),
            0,
            vbuf.cur_vertices as _,
        );
        check_gl_err();
    }
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
    use_line_shader(window, start, end);

    unsafe {
        // We reuse the rect VAO since it has no vertices associated to it.
        gl::BindVertexArray(window.gl.rect_vao);

        window.gl.draw_arrays(gl::LINES, 0, 2);
    }
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

pub fn swap_vbuf(a: &mut Vertex_Buffer, b: &mut Vertex_Buffer) -> bool {
    mem::swap(a, b);
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

fn get_mvp_matrix(
    window: &Render_Window_Handle,
    transform: &Transform2D,
    camera: &Transform2D,
) -> Matrix3<f32> {
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
    projection * view.get_matrix() * model.get_matrix()
}

fn use_rect_ws_shader(
    window: &mut Render_Window_Handle,
    color: Color,
    rect: &Rect<f32>,
    transform: &Transform2D,
    camera: &Transform2D,
) {
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
    let mvp = get_mvp_matrix(window, transform, camera);

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

        let (ww, wh) = inle_win::window::get_window_target_size(window);

        gl::Uniform2f(
            get_uniform_loc(window.gl.vbuf_shader, c_str!("win_half_size")),
            ww as f32 * 0.5,
            wh as f32 * 0.5,
        );

        gl::UniformMatrix3fv(
            get_uniform_loc(window.gl.vbuf_shader, c_str!("transform")),
            1,
            gl::FALSE,
            transform.get_matrix().as_slice().as_ptr(),
        );
    }
}

fn use_vbuf_ws_shader(
    window: &mut Render_Window_Handle,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    let mvp = get_mvp_matrix(window, transform, camera);
    unsafe {
        gl::UseProgram(window.gl.vbuf_ws_shader);

        gl::UniformMatrix3fv(
            get_uniform_loc(window.gl.vbuf_ws_shader, c_str!("mvp")),
            1,
            gl::FALSE,
            mvp.as_slice().as_ptr(),
        );
        check_gl_err();
    }
}

fn use_line_shader(window: &mut Render_Window_Handle, start: &Vertex, end: &Vertex) {
    let (ww, wh) = inle_win::window::get_window_target_size(window);
    let ww = ww as f32 * 0.5;
    let wh = wh as f32 * 0.5;
    unsafe {
        gl::UseProgram(window.gl.line_shader);

        gl::Uniform2fv(
            get_uniform_loc(window.gl.line_shader, c_str!("pos")),
            2,
            [
                (start.position.x - ww) / ww,
                (wh - start.position.y) / wh,
                (end.position.x - ww) / ww,
                (wh - end.position.y) / wh,
            ]
            .as_ptr(),
        );
        check_gl_err();

        gl::Uniform4fv(
            get_uniform_loc(window.gl.line_shader, c_str!("color")),
            2,
            [
                start.color.x,
                start.color.y,
                start.color.z,
                start.color.w,
                end.color.x,
                end.color.y,
                end.color.z,
                end.color.w,
            ]
            .as_ptr(),
        );
        check_gl_err();
    }
}

fn use_circle_shader(window: &mut Render_Window_Handle, color: Color, circle: shapes::Circle) {
    let transform = Transform2D::from_pos_rot_scale(
        circle.center,
        inle_math::angle::Angle::default(),
        v2!(circle.radius, circle.radius) * 2.0,
    );

    let (ww, wh) = inle_win::window::get_window_target_size(window);
    let ww = ww as f32 * 0.5;
    let wh = wh as f32 * 0.5;

    unsafe {
        gl::UseProgram(window.gl.circle_shader);

        gl::UniformMatrix3fv(
            get_uniform_loc(window.gl.circle_shader, c_str!("transform")),
            1,
            gl::FALSE,
            transform.get_matrix().as_slice().as_ptr(),
        );

        gl::Uniform2f(
            get_uniform_loc(window.gl.circle_shader, c_str!("win_half_size")),
            ww,
            wh,
        );

        gl::Uniform4f(
            get_uniform_loc(window.gl.circle_shader, c_str!("color")),
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        );
        check_gl_err();
    }
}

fn use_circle_ws_shader(
    window: &mut Render_Window_Handle,
    color: Color,
    circle: shapes::Circle,
    camera: &Transform2D,
) {
    let transform = Transform2D::from_pos_rot_scale(
        circle.center,
        inle_math::angle::Angle::default(),
        v2!(circle.radius, circle.radius) * 2.0,
    );

    let mvp = get_mvp_matrix(window, &transform, camera);

    unsafe {
        gl::UseProgram(window.gl.circle_ws_shader);

        gl::UniformMatrix3fv(
            get_uniform_loc(window.gl.circle_ws_shader, c_str!("mvp")),
            1,
            gl::FALSE,
            mvp.as_slice().as_ptr(),
        );

        gl::Uniform4f(
            get_uniform_loc(window.gl.circle_ws_shader, c_str!("color")),
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
