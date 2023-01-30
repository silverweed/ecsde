use crate::backend_common::alloc::Buffer_Allocators;
use crate::backend_common::misc::*;
use crate::render::get_vp_matrix;
use crate::render::gl::Uniform_Buffer;
use gl::types::*;
use inle_alloc::temp;
use inle_common::colors::Color;
use inle_math::rect::{Rect, Rectf, Recti};
use inle_math::transform::Transform2D;
use inle_math::vector::{Vec2f, Vec2i};
use inle_win::window::Window_Handle;
use std::collections::HashMap;
use std::{mem, ptr, str};

// I'd like this to be in backend_common::misc, but fuck if I understand
// how you're supposed to make it visible. It should be as simple as a
// #[macro_export] in misc.rs and then using it from here, but no matter how
// many #[macro_use] I put where, it's not being seen. So thanks a lot Rust,
// I'll just copy-paste this macro where it's needed until I have time and
// willpower to figure this garbage out.
macro_rules! glcheck {
    ($expr: expr) => {{
        let res = $expr;
        check_gl_err();
        res
    }};
}

pub struct Render_Window_Handle {
    window: Window_Handle,
    viewport: Recti,
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

pub fn create_render_window(mut window: Window_Handle) -> Render_Window_Handle {
    glcheck!(gl::load_with(|symbol| inle_win::window::get_gl_handle(
        &mut window,
        symbol
    )));
    let win_size = inle_win::window::get_window_target_size(&window);
    Render_Window_Handle {
        window,
        viewport: Recti::new(0, 0, win_size.0 as _, win_size.1 as _),
        gl: init_gl(),
        temp_allocator: temp::Temp_Allocator::with_capacity(inle_common::units::megabytes(10)),
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

    pub circle_shader: GLuint,

    pub text_shader: GLuint,

    pub uniform_buffers: HashMap<&'static std::ffi::CStr, Uniform_Buffer>,

    #[cfg(debug_assertions)]
    pub n_draw_calls_this_frame: u32,
    #[cfg(debug_assertions)]
    pub n_draw_calls_prev_frame: u32,
}

impl Gl {
    pub const fn n_rect_indices(&self) -> GLsizei {
        RECT_INDICES.len() as _
    }

    pub const fn rect_indices_type(&self) -> GLenum {
        gl::UNSIGNED_INT
    }

    pub fn draw_arrays(&mut self, primitive: GLenum, first: GLint, count: GLsizei) {
        unsafe {
            glcheck!(gl::DrawArrays(primitive, first, count));
        }

        #[cfg(debug_assertions)]
        {
            self.n_draw_calls_this_frame += 1;
        }
    }

    pub fn draw_indexed(&mut self, indices: GLsizei, indices_type: GLenum) {
        unsafe {
            glcheck!(gl::DrawElements(
                gl::TRIANGLES,
                indices,
                indices_type,
                ptr::null()
            ));
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

    gl.rect_shader = create_shader_from!("rect", "basic_color");
    gl.line_shader = create_shader_from!("line", "vbuf");
    gl.vbuf_shader = create_shader_from!("vbuf", "vbuf");
    gl.vbuf_texture_shader = create_shader_from!("vbuf", "vbuf_texture");
    gl.circle_shader = create_shader_from!("rect", "circle");
    gl.text_shader = create_shader_from!("vbuf", "msdf");

    unsafe {
        glcheck!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));
        glcheck!(gl::Enable(gl::BLEND));

        glcheck!(gl::FrontFace(gl::CW));
        glcheck!(gl::Enable(gl::CULL_FACE));
    }

    #[cfg(debug_assertions)]
    unsafe {
        glcheck!(gl::Enable(gl::DEBUG_OUTPUT));
        glcheck!(gl::DebugMessageCallback(Some(gl_msg_callback), ptr::null()));
    }

    gl
}

fn fill_rect_buffers(gl: &mut Gl) {
    let (mut vao, mut ebo) = (0, 0);
    unsafe {
        glcheck!(gl::GenBuffers(1, &mut ebo));
        debug_assert!(ebo != 0);

        glcheck!(gl::GenVertexArrays(1, &mut vao));
        debug_assert!(vao != 0);

        glcheck!(gl::BindVertexArray(vao));

        glcheck!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo));

        glcheck!(gl::BufferStorage(
            gl::ELEMENT_ARRAY_BUFFER,
            (RECT_INDICES.len() * mem::size_of::<GLuint>()) as _,
            RECT_INDICES.as_ptr() as *const _,
            0,
        ));
    }

    gl.rect_vao = vao;
    gl.rect_ebo = ebo;
}

#[inline(always)]
pub fn shutdown(window: &mut Render_Window_Handle) {
    window.gl.buffer_allocators.destroy();

    let buf_ids = window
        .gl
        .uniform_buffers
        .values()
        .map(|buf| buf.id)
        .collect::<Vec<_>>();
    unsafe {
        glcheck!(gl::DeleteBuffers(buf_ids.len() as _, buf_ids.as_ptr()));
    }

    for buf in window.gl.uniform_buffers.values_mut() {
        unsafe {
            std::alloc::dealloc(buf.mem, buf.layout);
        }
    }
}

pub fn recreate_render_window(window: &mut Render_Window_Handle) {
    glcheck!(gl::load_with(|symbol| inle_win::window::get_gl_handle(
        &mut window.window,
        symbol
    )));
}

pub fn set_clear_color(_window: &mut Render_Window_Handle, color: Color) {
    unsafe {
        glcheck!(gl::ClearColor(
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        ));
    }
}

pub fn clear(_window: &mut Render_Window_Handle) {
    unsafe {
        glcheck!(gl::Clear(gl::COLOR_BUFFER_BIT));
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

    window.viewport = viewport;

    unsafe {
        glcheck!(gl::Viewport(
            viewport.x,
            viewport.y,
            viewport.width,
            viewport.height
        ));
    }
}

/// Converts screen coordinates (where (0,0) is top-left of the _viewport_) to world coordinates
/// as seen from `camera`.
pub fn unproject_screen_pos(
    screen_pos: Vec2i,
    window: &Render_Window_Handle,
    camera: &Transform2D,
) -> Vec2f {
    let vp = get_vp_matrix(window, camera);
    let ndc = v2!(
        2. * (screen_pos.x as f32 - window.viewport.x as f32) / window.viewport.width as f32 - 1.,
        1. - 2. * (screen_pos.y as f32 - window.viewport.y as f32) / window.viewport.height as f32,
    );

    (&vp.inverse() * v3!(ndc.x, ndc.y, 1.0)).into()
}

/// Converts world coordinates to viewport coordinates (i.e. screen coordinates where (0,0) is the
/// top-left of the viewport).
pub fn project_world_pos(
    world_pos: Vec2f,
    window: &Render_Window_Handle,
    camera: &Transform2D,
) -> Vec2i {
    let vp = get_vp_matrix(window, camera);
    let clip = &vp * v3!(world_pos.x, world_pos.y, 1.0);
    let ndc = v2!(clip.x / clip.z, -clip.y / clip.z);
    let (win_w, win_h) = inle_win::window::get_window_target_size(window);
    v2!(
        (ndc.x + 1.) * 0.5 * win_w as f32,
        (ndc.y + 1.) * 0.5 * win_h as f32,
    )
    .into()
}

#[inline(always)]
pub fn start_new_frame(window: &mut Render_Window_Handle) {
    window.gl.buffer_allocators.dealloc_all_temp();
    unsafe {
        window.temp_allocator.dealloc_all();
    }

    #[cfg(debug_assertions)]
    {
        window.gl.n_draw_calls_prev_frame = window.gl.n_draw_calls_this_frame;
        window.gl.n_draw_calls_this_frame = 0;
    }
}

#[inline(always)]
pub fn n_draw_calls_prev_frame(window: &Render_Window_Handle) -> u32 {
    #[cfg(debug_assertions)]
    {
        window.gl.n_draw_calls_prev_frame
    }
    #[cfg(not(debug_assertions))]
    {
        0
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
