use crate::common::vector::Vec2i;
use glfw;
use glfw::Context;

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
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn create_window(
    args: &Create_Window_Args,
    target_size: (u32, u32),
    title: &str,
) -> Window_Handle {
    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
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
    // @Incomplete: vsync, etc

    Window_Handle {
        handle: window,
        target_size,
        vsync: args.vsync,
        glfw,
        event_receiver: events,
    }
}

pub fn has_vsync<W: AsRef<Window_Handle>>(window: &W) -> bool {
    window.as_ref().vsync
}

pub fn set_vsync<W: AsMut<Window_Handle>>(window: &mut W, vsync: bool) {
    let window = window.as_mut();
    window.vsync = vsync;
    window.glfw.set_swap_interval(if vsync {
        glfw::SwapInterval::Sync(1)
    } else {
        glfw::SwapInterval::None
    });
}

pub fn display<W: AsMut<Window_Handle>>(window: &mut W) {
    window.as_mut().handle.swap_buffers();
}

pub fn get_window_target_size<W: AsRef<Window_Handle>>(window: &W) -> (u32, u32) {
    window.as_ref().target_size
}

pub fn get_window_real_size<W: AsRef<Window_Handle>>(window: &W) -> (u32, u32) {
    let (x, y) = window.as_ref().handle.get_size();
    (x as _, y as _)
}

pub fn poll_event<W: AsMut<Window_Handle>>(window: &mut W) -> Option<Event> {
    // @Incomplete
    let window = window.as_mut();
    window.glfw.poll_events();
    glfw::flush_messages(&window.event_receiver)
        .next()
        .map(|(_, evt)| evt)
}

pub fn raw_mouse_pos_in_window<W: AsRef<Window_Handle>>(window: &W) -> Vec2i {
    let (x, y) = window.as_ref().handle.get_cursor_pos();
    debug_assert!(x < std::i32::MAX as f64);
    debug_assert!(y < std::i32::MAX as f64);
    v2!(x as i32, y as i32)
}

pub fn set_key_repeat_enabled<W: AsMut<Window_Handle>>(window: &mut W, enabled: bool) {
    // @Incomplete
    window.as_mut().handle.set_key_polling(!enabled);
}
