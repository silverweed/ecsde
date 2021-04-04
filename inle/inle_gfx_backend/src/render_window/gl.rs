use crate::backend_common::alloc::Buffer_Allocators;
use crate::backend_common::misc::check_gl_err;
use gl::types::*;
use inle_alloc::temp;
use inle_common::colors::Color;
use inle_math::rect::{Rect, Rectf};
use inle_math::transform::Transform2D;
use inle_math::vector::{Vec2f, Vec2i};
use inle_win::window::Window_Handle;
use std::{mem, ptr, str};

pub struct Render_Window_Handle {
    window: Window_Handle,
    pub gl: Gl,
    pub temp_allocator: temp::Temp_Allocator,
}

impl AsRef<Window_Handle> for Render_Window_Handle {
    fn as_ref(&self) -> &Window_Handle {
        &self.window
    }
}

impl AsMut<Window_Handle> for Render_Window_Handle {
    fn as_mut(&mut self) -> &mut Window_Handle {
        &mut self.window
    }
}

const RECT_INDICES: [GLuint; 6] = [0, 1, 2, 2, 3, 0];

#[derive(Default)]
pub struct Gl {
    pub buffer_allocators: Buffer_Allocators,

    pub rect_vao: GLuint,
    pub rect_ebo: GLuint,
    pub rect_shader: GLuint,

    pub line_shader: GLuint,

    pub vbuf_shader: GLuint,
    pub vbuf_texture_shader: GLuint,

    pub circle_vao: GLuint,
    pub circle_vbo: GLuint,
    pub circle_shader: GLuint,

    pub text_shader: GLuint,

    #[cfg(debug_assertions)]
    pub n_draw_calls_this_frame: u32,
}

impl Gl {
    pub const fn n_rect_indices(&self) -> GLsizei {
        RECT_INDICES.len() as _
    }

    pub const fn rect_indices_type(&self) -> GLenum {
        gl::UNSIGNED_INT
    }

    pub const fn n_circle_vertices(&self) -> GLsizei {
        CIRCLE_VERTICES.len() as _
    }

    pub fn draw_arrays(&mut self, primitive: GLenum, first: GLint, count: GLsizei) {
        unsafe {
            gl::DrawArrays(primitive, first, count);
            check_gl_err();
        }

        #[cfg(debug_assertions)]
        {
            self.n_draw_calls_this_frame += 1;
        }
    }

    pub fn draw_indexed(&mut self, indices: GLsizei, indices_type: GLenum) {
        unsafe {
            gl::DrawElements(gl::TRIANGLES, indices, indices_type, ptr::null());
            check_gl_err();
        }

        #[cfg(debug_assertions)]
        {
            self.n_draw_calls_this_frame += 1;
        }
    }
}

macro_rules! create_shader_from {
    ($vert: expr, $frag: expr) => {{
        const VERT_SHADER_SRC: &str =
            include_str!(concat!("./gl/builtin_shaders/", $vert, ".vert"));
        const FRAG_SHADER_SRC: &str =
            include_str!(concat!("./gl/builtin_shaders/", $frag, ".frag"));

        crate::render::new_shader_internal(
            VERT_SHADER_SRC.as_bytes(),
            FRAG_SHADER_SRC.as_bytes(),
            concat!($vert, "+", $frag),
        )
    }};
}

fn init_gl() -> Gl {
    let mut gl = Gl::default();

    fill_rect_buffers(&mut gl);
    fill_circle_buffers(&mut gl);

    gl.rect_shader = create_shader_from!("rect", "basic_color");
    gl.line_shader = create_shader_from!("line", "vbuf");
    gl.vbuf_shader = create_shader_from!("vbuf", "vbuf");
    gl.vbuf_texture_shader = create_shader_from!("vbuf", "vbuf_texture");
    gl.circle_shader = create_shader_from!("circle", "basic_color");
    gl.text_shader = create_shader_from!("vbuf", "msdf");

    unsafe {
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::BLEND);
    }

    #[cfg(debug_assertions)]
    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        check_gl_err();
        gl::DebugMessageCallback(Some(gl_msg_callback), ptr::null());
    }

    gl
}

fn fill_rect_buffers(gl: &mut Gl) {
    let (mut vao, mut ebo) = (0, 0);
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut ebo);

        debug_assert!(vao != 0);
        debug_assert!(ebo != 0);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferStorage(
            gl::ELEMENT_ARRAY_BUFFER,
            (RECT_INDICES.len() * mem::size_of::<GLuint>()) as _,
            RECT_INDICES.as_ptr() as *const _,
            0,
        );
    }

    gl.rect_vao = vao;
    gl.rect_ebo = ebo;
}

fn fill_circle_buffers(gl: &mut Gl) {
    const LOC_IN_POS: GLuint = 0;

    let (mut vao, mut vbo) = (0, 0);
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        debug_assert!(vao != 0);
        debug_assert!(vbo != 0);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferStorage(
            gl::ARRAY_BUFFER,
            (CIRCLE_VERTICES.len() * mem::size_of::<GLfloat>()) as _,
            CIRCLE_VERTICES.as_ptr() as *const _,
            0,
        );

        gl::VertexAttribPointer(
            LOC_IN_POS,
            2,
            gl::FLOAT,
            gl::FALSE,
            2 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(LOC_IN_POS);
    }

    gl.circle_vao = vao;
    gl.circle_vbo = vbo;
}

pub fn create_render_window(mut window: Window_Handle) -> Render_Window_Handle {
    gl::load_with(|symbol| inle_win::window::get_gl_handle(&mut window, symbol));
    Render_Window_Handle {
        window,
        gl: init_gl(),
        temp_allocator: temp::Temp_Allocator::with_capacity(inle_common::units::megabytes(10)),
    }
}

#[inline(always)]
pub fn shutdown(window: &mut Render_Window_Handle) {
    window.gl.buffer_allocators.destroy();
}

pub fn recreate_render_window(window: &mut Render_Window_Handle) {
    gl::load_with(|symbol| inle_win::window::get_gl_handle(&mut window.window, symbol));
}

pub fn set_clear_color(_window: &mut Render_Window_Handle, color: Color) {
    unsafe {
        gl::ClearColor(
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        );
    }
}

pub fn clear(_window: &mut Render_Window_Handle) {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

pub fn set_viewport(window: &mut Render_Window_Handle, viewport: &Rectf, _view_rect: &Rectf) {
    let win_size = inle_win::window::get_window_real_size(window);
    let width = win_size.0 as f32;
    let height = win_size.1 as f32;

    // de-normalize the viewport
    let viewport = Rect::new(
        (0.5 + width * viewport.x) as i32,
        (0.5 + height * viewport.y) as i32,
        (0.5 + width * viewport.width) as i32,
        (0.5 + height * viewport.height) as i32,
    );

    unsafe {
        gl::Viewport(viewport.x, viewport.y, viewport.width, viewport.height);
    }
}

pub fn raw_unproject_screen_pos(
    screen_pos: Vec2i,
    _window: &Render_Window_Handle,
    camera: &Transform2D,
) -> Vec2f {
    (*camera * Vec2f::from(screen_pos)).into()
}

pub fn raw_project_world_pos(
    world_pos: Vec2f,
    window: &Render_Window_Handle,
    camera: &Transform2D,
) -> Vec2i {
    let pos_cam_space = camera.inverse() * world_pos;
    //let screen_pos = window
    //.raw_handle()
    //.map_coords_to_pixel_current_view(pos_cam_space);
    //screen_pos.into()
    pos_cam_space.into()
}

#[inline(always)]
pub fn start_new_frame(window: &mut Render_Window_Handle) {
    window.gl.buffer_allocators.dealloc_all_temp();
    unsafe {
        window.temp_allocator.dealloc_all();
    }

    #[cfg(debug_assertions)]
    {
        window.gl.n_draw_calls_this_frame = 0;
    }
}

#[cfg(debug_assertions)]
extern "system" fn gl_msg_callback(
    _source: GLenum,
    typ: GLenum,
    _id: GLuint,
    _severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    _user_param: *mut std::ffi::c_void,
) {
    let message: &[u8] =
        unsafe { std::slice::from_raw_parts(message as *const u8, length as usize) };
    if typ == gl::DEBUG_TYPE_ERROR {
        lerr!("[GL ERROR] {}\n", str::from_utf8(message).unwrap());
    } else {
        lverbose!("[GL MSG] {}\n", str::from_utf8(message).unwrap());
    }
}

const N_CIRCLE_POINTS: usize = 32;
const CIRCLE_VERTICES: [GLfloat; 2 * (N_CIRCLE_POINTS + 2)] = [
    0., 0., 0.500000, 0.000000, 0.490393, 0.097545, 0.461940, 0.191342, 0.415735, 0.277785,
    0.353553, 0.353553, 0.277785, 0.415735, 0.191342, 0.461940, 0.097545, 0.490393, -0.000000,
    0.500000, -0.097545, 0.490393, -0.191342, 0.461940, -0.277785, 0.415735, -0.353553, 0.353553,
    -0.415735, 0.277785, -0.461940, 0.191342, -0.490393, 0.097545, -0.500000, -0.000000, -0.490393,
    -0.097545, -0.461940, -0.191342, -0.415735, -0.277785, -0.353553, -0.353553, -0.277785,
    -0.415735, -0.191342, -0.461940, -0.097545, -0.490393, 0.000000, -0.500000, 0.097545,
    -0.490393, 0.191342, -0.461940, 0.277785, -0.415735, 0.353553, -0.353553, 0.415735, -0.277785,
    0.461940, -0.191342, 0.490393, -0.097545, 0.5, 0.,
];
