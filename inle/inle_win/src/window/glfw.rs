use glfw::Context;
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};

pub enum Event {
    Window(glfw::WindowEvent),
    Joystick {
        joy_id: glfw::JoystickId,
        connected: bool,
        guid: Option<Box<str>>,
    },
}

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
    pub event_receiver: Receiver<(f64, glfw::WindowEvent)>,
    events_buffer: VecDeque<Event>,
    joystick_events: Receiver<Event>,

    /// We keep track of these ourselves because going through GLFW queries may be costly
    // Cursor pos in pixels (relative to the window)
    cursor_pos: (f64, f64),
    // Size of the framebuffer
    real_size: (u32, u32),
}

#[cfg(feature = "gfx-gl")]
pub fn get_gl_handle(window: &mut Window_Handle, s: &'static str) -> *const std::ffi::c_void {
    window.handle.get_proc_address(s)
}

#[must_use]
fn create_joystick_callback(
    glfw: &glfw::Glfw,
) -> (
    glfw::JoystickCallback<(Sender<Event>, glfw::Glfw)>,
    Receiver<Event>,
) {
    let (joy_evt_sender, joy_evt_recv) = channel();
    (
        glfw::Callback {
            f: |joy_id, joy_evt, (joy_evt_sender, glfw): &(Sender<Event>, glfw::Glfw)| {
                ldebug!("Joystick {:?} connected", joy_id);

                let joy = glfw::Joystick {
                    id: joy_id,
                    glfw: glfw.clone(),
                };
                joy_evt_sender
                    .send(Event::Joystick {
                        joy_id,
                        connected: joy_evt == glfw::JoystickEvent::Connected,
                        guid: joy.get_guid().map(|s| s.into_boxed_str()),
                    })
                    .unwrap_or_else(|err| {
                        lerr!("Failed to send Joystick event from callback: {:?}", err)
                    });
            },
            data: (joy_evt_sender, glfw.clone()),
        },
        joy_evt_recv,
    )
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn create_window(
    args: &Create_Window_Args,
    target_size: (u32, u32),
    title: &str,
) -> Window_Handle {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 4));
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
    window.set_scroll_polling(true);

    // Without the following 2, maximizing the just-opened window will lose focus on some platforms (e.g.
    // with Awesome WM).
    window.set_focus_polling(true);
    window.set_focus_on_show(true);

    // @Cleanup: needed?
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_size_limits(Some(1), Some(1), None, None);
    // We handle aspect ratio ourselves
    window.set_aspect_ratio(glfw::ffi::DONT_CARE as u32, glfw::ffi::DONT_CARE as u32);
    window.set_mouse_button_polling(true);

    let (joy_cb, joy_evt_recv) = create_joystick_callback(&glfw);
    glfw.set_joystick_callback(Some(joy_cb));

    // @Incomplete: vsync, etc

    Window_Handle {
        handle: window,
        target_size,
        vsync: args.vsync,
        glfw,
        event_receiver: events,
        events_buffer: VecDeque::with_capacity(8),
        cursor_pos: (0., 0.),
        real_size: (1, 1),
        joystick_events: joy_evt_recv,
    }
}

pub fn recreate_window(window: &mut Window_Handle) {
    window.glfw.set_joystick_callback::<()>(None);
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let (joy_cb, joy_evt_recv) = create_joystick_callback(&glfw);
    glfw.set_joystick_callback(Some(joy_cb));
    window.glfw = glfw;
    window.joystick_events = joy_evt_recv;
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
    window.real_size
}

pub fn prepare_poll_events(window: &mut Window_Handle) {
    window.glfw.poll_events();
    window.events_buffer.clear();
    for (_, evt) in glfw::flush_messages(&window.event_receiver) {
        window.events_buffer.push_back(Event::Window(evt));
    }

    while let Ok(evt) = window.joystick_events.try_recv() {
        window.events_buffer.push_back(evt);
    }
}

pub fn poll_event(window: &mut Window_Handle) -> Option<Event> {
    let evt = window.events_buffer.pop_front();
    if let Some(Event::Window(glfw::WindowEvent::FramebufferSize(x, y))) = evt {
        window.real_size = (x as u32, y as u32);
    }
    evt
}

pub fn set_key_repeat_enabled(_window: &mut Window_Handle, _enabled: bool) {}
