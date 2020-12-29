use gl::types::*;
use inle_common::colors::Color;
use inle_math::rect::{Rect, Rectf};
use inle_math::transform::Transform2D;
use inle_math::vector::{Vec2f, Vec2i};
use inle_win::window::Window_Handle;
use std::ffi::{c_void, CString};
use std::{mem, ptr, str};

pub struct Render_Window_Handle {
    window: Window_Handle,
    pub gl: Gl,
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
    pub rect_vao: GLuint,
    pub rect_ebo: GLuint,
    pub rect_shader: GLuint,
    pub rect_ws_shader: GLuint,
    pub line_shader: GLuint,
    pub vbuf_shader: GLuint,

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

    pub fn draw_indexed(&mut self, indices: GLsizei, indices_type: GLenum) {
        unsafe {
            gl::DrawElements(gl::TRIANGLES, indices, indices_type, ptr::null());
        }

        #[cfg(debug_assertions)]
        {
            self.n_draw_calls_this_frame += 1;
        }
    }
}

fn init_gl() -> Gl {
    let mut gl = Gl::default();

    fill_rect_buffers(&mut gl);
    init_rect_shader(&mut gl);
    init_rect_ws_shader(&mut gl);
    init_line_shader(&mut gl);
    init_vbuf_shader(&mut gl);

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
    const LOC_IN_POS: GLuint = 0;

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

        gl::VertexAttribPointer(
            LOC_IN_POS,
            2,
            gl::FLOAT,
            gl::FALSE,
            2 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(LOC_IN_POS);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    gl.rect_vao = vao;
    gl.rect_ebo = ebo;
}

const GL_TRUE: GLint = gl::TRUE as _;
const GL_FALSE: GLint = gl::FALSE as _;

macro_rules! create_shader_from {
    ($vert: expr, $frag: expr) => {{
        const VERT_SHADER_SRC: &str =
            include_str!(concat!("./gl/builtin_shaders/", $vert, ".vert"));
        const FRAG_SHADER_SRC: &str =
            include_str!(concat!("./gl/builtin_shaders/", $frag, ".frag"));

        create_shader(VERT_SHADER_SRC, FRAG_SHADER_SRC, concat!($vert, "+", $frag))
    }};
}

fn init_rect_shader(gl: &mut Gl) {
    gl.rect_shader = create_shader_from!("screen_rect", "basic_color");
}

fn init_rect_ws_shader(gl: &mut Gl) {
    gl.rect_ws_shader = create_shader_from!("ws_rect", "basic_color");
}

fn init_line_shader(gl: &mut Gl) {
    gl.line_shader = create_shader_from!("line", "line");
}

fn init_vbuf_shader(gl: &mut Gl) {
    gl.vbuf_shader = create_shader_from!("vbuf", "vbuf");
}

fn create_shader(vertex_src: &str, fragment_src: &str, shader_src: &str) -> GLuint {
    unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_vert = CString::new(vertex_src.as_bytes()).unwrap();

        gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);

        const INFO_LOG_CAP: GLint = 512;
        let mut info_log = Vec::with_capacity(INFO_LOG_CAP as usize);
        info_log.set_len(INFO_LOG_CAP as usize - 1); // subtract 1 to skip the trailing null character

        let mut success = GL_FALSE;
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        let mut info_len = 0;
        if success != GL_TRUE {
            gl::GetShaderInfoLog(
                vertex_shader,
                INFO_LOG_CAP,
                &mut info_len,
                info_log.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "Vertex shader `{}` failed to compile:\n----------\n{}\n-----------",
                shader_src,
                str::from_utf8(&info_log[..info_len as usize]).unwrap()
            );
        }

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_frag = CString::new(fragment_src.as_bytes()).unwrap();

        gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);

        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != GL_TRUE {
            gl::GetShaderInfoLog(
                fragment_shader,
                INFO_LOG_CAP,
                &mut info_len,
                info_log.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "Fragment shader `{}` failed to compile:\n----------\n{}\n-----------",
                shader_src,
                str::from_utf8(&info_log[..info_len as usize]).unwrap()
            );
        }

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != GL_TRUE {
            gl::GetProgramInfoLog(
                shader_program,
                INFO_LOG_CAP,
                &mut info_len,
                info_log.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "Shader `{}` failed to link:\n----------\n{}\n-----------",
                shader_src,
                str::from_utf8(&info_log[..info_len as usize]).unwrap()
            );
        }
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        debug_assert!(shader_program != 0);
        ldebug!(
            "Shader `{}` ({}) linked successfully.",
            shader_src,
            shader_program
        );

        shader_program
    }
}

pub fn create_render_window(mut window: Window_Handle) -> Render_Window_Handle {
    gl::load_with(|symbol| inle_win::window::get_gl_handle(&mut window, symbol));
    Render_Window_Handle {
        window,
        gl: init_gl(),
    }
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
    _screen_pos: Vec2i,
    _window: &Render_Window_Handle,
    _camera: &Transform2D,
) -> Vec2f {
    Vec2f::default()
}

pub fn raw_project_world_pos(
    _world_pos: Vec2f,
    _window: &Render_Window_Handle,
    _camera: &Transform2D,
) -> Vec2i {
    Vec2i::default()
}

#[inline(always)]
pub fn start_new_frame(_window: &mut Render_Window_Handle) {
    #[cfg(debug_assertions)]
    {
        _window.gl.n_draw_calls_this_frame = 0;
    }
}

extern "system" fn gl_msg_callback(
    _source: GLenum,
    typ: GLenum,
    _id: GLuint,
    _severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    _user_param: *mut c_void,
) {
    let message: &[u8] =
        unsafe { std::slice::from_raw_parts(message as *const u8, length as usize) };
    lerr!(
        "GL MSG: {} message = {}\n",
        if typ == gl::DEBUG_TYPE_ERROR {
            "** GL ERROR **"
        } else {
            ""
        },
        str::from_utf8(message).unwrap()
    );
}

// @Cleanup: don't duplicate this code from render/gl.rs! Share it!
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
