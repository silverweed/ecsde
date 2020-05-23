#![cfg(feature = "use-sfml")]

use crate::common::vector::Vec2i;
use sfml::window as sfwin;

pub type Event = sfml::window::Event;

pub struct Create_Window_Args {
    pub vsync: bool,
}

impl Default for Create_Window_Args {
    fn default() -> Self {
        Create_Window_Args { vsync: true }
    }
}

pub struct Window_Handle {
    handle: sfwin::Window,
    target_size: (u32, u32),
    vsync: bool,
}

impl Window_Handle {
    #[inline(always)]
    pub fn raw_handle(&self) -> &sfwin::Window {
        &self.handle
    }

    #[inline(always)]
    pub fn raw_handle_mut(&mut self) -> &mut sfwin::Window {
        &mut self.handle
    }
}

pub fn create_window(
    create_args: &Create_Window_Args,
    target_size: (u32, u32),
    title: &str,
) -> Window_Handle {
    let mut window = sfwin::Window::new(
        target_size,
        title,
        sfwin::Style::DEFAULT,
        &sfwin::ContextSettings::default(),
    );
    window.set_vertical_sync_enabled(create_args.vsync);
    window.set_key_repeat_enabled(false);
    Window_Handle {
        handle: window,
        target_size,
        vsync: create_args.vsync,
    }
}

pub fn destroy_window(window: &mut Window_Handle) {
    window.handle.close();
}

pub fn get_window_target_size(window: &Window_Handle) -> (u32, u32) {
    window.target_size
}

pub fn has_vsync(window: &Window_Handle) -> bool {
    window.vsync
}

pub fn set_vsync(window: &mut Window_Handle, vsync: bool) {
    window.handle.set_vertical_sync_enabled(vsync);
    window.vsync = vsync;
}

pub fn raw_mouse_pos_in_window(window: &Window_Handle) -> Vec2i {
    window.handle.mouse_position().into()
}

pub fn get_window_real_size(window: &Window_Handle) -> (u32, u32) {
    let v = window.handle.size();
    (v.x, v.y)
}

pub fn set_key_repeat_enabled(window: &mut Window_Handle, enabled: bool) {
    window.handle.set_key_repeat_enabled(enabled);
}

pub fn poll_event(window: &mut Window_Handle) -> Option<Event> {
    window.handle.poll_event()
}
