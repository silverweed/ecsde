use glfw::Context;
use inle_math::vector::Vec2i;
use std::collections::VecDeque;

pub type Event = glfw::WindowEvent;

pub struct Create_Window_Args {
    pub vsync: bool,
}

impl Default for Create_Window_Args {
    fn default() -> Self {
        Create_Window_Args { vsync: true }
    }
}

type Window_Type = glfw::Window;

pub struct Window_Handle {
    handle: Window_Type,
    target_size: (u32, u32),
    vsync: bool,
    pub glfw: glfw::Glfw,
    pub event_receiver: std::sync::mpsc::Receiver<(f64, Event)>,
    events_buffer: VecDeque<Event>,
}

#[cfg(feature = "gfx-gl")]
pub fn get_gl_handle(window: &mut Window_Handle, s: &'static str) -> *const std::ffi::c_void {
    window.handle.get_proc_address(s)
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn create_window(
    args: &Create_Window_Args,
    target_size: (u32, u32),
    title: &str,
) -> Window_Handle {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // @Incomplete: allow setting mode?
    let (mut window, events) = glfw
        .create_window(
            target_size.0,
            target_size.1,
            title,
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    // @Cleanup: needed?
    window.set_framebuffer_size_polling(true);
    // @Incomplete: vsync, etc

    Window_Handle {
        handle: window,
        target_size,
        vsync: args.vsync,
        glfw,
        event_receiver: events,
        events_buffer: VecDeque::with_capacity(8),
    }
}

pub fn has_vsync(window: &Window_Handle) -> bool {
    window.vsync
}

pub fn set_vsync(window: &mut Window_Handle, vsync: bool) {
    window.vsync = vsync;
    window.glfw.set_swap_interval(if vsync {
        glfw::SwapInterval::Sync(1)
    } else {
        glfw::SwapInterval::None
    });
}

pub fn display(window: &mut Window_Handle) {
    window.handle.swap_buffers();
}

pub fn get_window_target_size(window: &Window_Handle) -> (u32, u32) {
    window.target_size
}

pub fn get_window_real_size(window: &Window_Handle) -> (u32, u32) {
    let (x, y) = window.handle.get_size();
    (x as _, y as _)
}

pub fn prepare_poll_events(window: &mut Window_Handle) {
    window.glfw.poll_events();
    window.events_buffer.clear();
    for (_, evt) in glfw::flush_messages(&window.event_receiver) {
        window.events_buffer.push_back(evt);
    }
}

pub fn poll_event(window: &mut Window_Handle) -> Option<Event> {
    window.events_buffer.pop_front()
}

pub fn raw_mouse_pos_in_window(window: &Window_Handle) -> Vec2i {
    let (x, y) = window.handle.get_cursor_pos();
    debug_assert!(x < std::i32::MAX as f64);
    debug_assert!(y < std::i32::MAX as f64);
    v2!(x as i32, y as i32)
}

pub fn set_key_repeat_enabled(_window: &mut Window_Handle, _enabled: bool) {}
