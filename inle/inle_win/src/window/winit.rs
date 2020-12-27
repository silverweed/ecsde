#![cfg(feature = "win-winit")]

use inle_math::vector::Vec2i;
use winit::event_loop::EventLoop;
use winit::platform::run_return::EventLoopExtRunReturn;

type Event_User_Type = ();
pub type Event<'a> = winit::event::Event<'a, Event_User_Type>;

pub struct Create_Window_Args;

impl Default for Create_Window_Args {
    fn default() -> Self {
        Create_Window_Args {}
    }
}

type Window_Type = winit::window::Window;

pub struct Window_Handle {
    handle: Window_Type,
    event_loop: EventLoop<Event_User_Type>,
    target_size: (u32, u32),
    raw_mpos: Vec2i,
}

impl Window_Handle {
    #[inline(always)]
    pub fn raw_handle(&self) -> &Window_Type {
        &self.handle
    }

    #[inline(always)]
    pub fn raw_handle_mut(&mut self) -> &mut Window_Type {
        &mut self.handle
    }
}

pub fn create_window(
    _create_args: &Create_Window_Args,
    target_size: (u32, u32),
    title: &str,
) -> Window_Handle {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_resizable(true)
        // @Note: is PhysicalSize the best choice?
        .with_inner_size(winit::dpi::PhysicalSize::new(target_size.0, target_size.1))
        .with_title(title)
        .with_decorations(true)
        .build(&event_loop)
        .unwrap();
    Window_Handle {
        handle: window,
        event_loop,
        target_size,
        raw_mpos: Vec2i::default(),
    }
}

pub fn get_window_target_size(window: &Window_Handle) -> (u32, u32) {
    window.target_size
}

pub fn has_vsync(_window: &Window_Handle) -> bool {
    false
}

pub fn set_vsync(_window: &mut Window_Handle, _vsync: bool) {}

pub fn raw_mouse_pos_in_window(window: &Window_Handle) -> Vec2i {
    // This must be set externally, which sucks! @Refactor!
    window.raw_mpos
}

pub fn get_window_real_size(window: &Window_Handle) -> (u32, u32) {
    let s = window.handle.inner_size();
    (s.width, s.height)
}

pub fn set_key_repeat_enabled(_window: &mut Window_Handle, _enabled: bool) {}

pub fn poll_event<'a, 'b>(window: &'b mut Window_Handle) -> Option<Event<'a>>
where
    'a: 'b,
{
    let mut returned_event = None;
    window
        .event_loop
        .run_return(|event, _window, control_flow| {
            returned_event = Some(event.clone());
            *control_flow = winit::event_loop::ControlFlow::Exit;
        });
    returned_event
}

pub fn display(window: &mut Window_Handle) {
    window.handle.request_redraw();
}
