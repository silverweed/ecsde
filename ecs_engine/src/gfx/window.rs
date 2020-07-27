use crate::common::vector::{Vec2f, Vec2i, Vec2u};

#[cfg(feature = "win-sfml")]
mod sfml;

#[cfg(feature = "win-glfw")]
mod glfw;

#[cfg(feature = "win-sfml")]
use self::sfml as backend;

#[cfg(feature = "win-glfw")]
use self::glfw as backend;

pub type Window_Handle = backend::Window_Handle;
pub type Create_Window_Args = backend::Create_Window_Args;
pub type Event = backend::Event;

impl AsRef<Window_Handle> for Window_Handle {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<Window_Handle> for Window_Handle {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn create_window(
    args: &Create_Window_Args,
    target_size: (u32, u32),
    title: &str,
) -> Window_Handle {
    backend::create_window(args, target_size, title)
}

pub fn has_vsync<W: AsRef<Window_Handle>>(window: &W) -> bool {
    backend::has_vsync(window.as_ref())
}

pub fn set_vsync<W: AsMut<Window_Handle>>(window: &mut W, vsync: bool) {
    backend::set_vsync(window.as_mut(), vsync);
}

pub fn display<W: AsMut<Window_Handle>>(window: &mut W) {
    backend::display(window.as_mut());
}

pub fn get_window_target_size<W: AsRef<Window_Handle>>(window: &W) -> (u32, u32) {
    backend::get_window_target_size(window.as_ref())
}

pub fn get_window_real_size<W: AsRef<Window_Handle>>(window: &W) -> (u32, u32) {
    backend::get_window_real_size(window.as_ref())
}

pub fn poll_event<W: AsMut<Window_Handle>>(window: &mut W) -> Option<Event> {
    backend::poll_event(window.as_mut())
}

pub fn mouse_pos_in_window<W: AsRef<Window_Handle>>(window: &W) -> Vec2i {
    let window = window.as_ref();
    let v = Vec2f::from(backend::raw_mouse_pos_in_window(window));

    let ts: Vec2u = backend::get_window_target_size(window).into();
    let target_ratio = ts.y as f32 / ts.x as f32;

    let rs: Vec2u = backend::get_window_real_size(window).into();
    let real_ratio = rs.y as f32 / rs.x as f32;

    let ratio = Vec2f::from(rs) / Vec2f::from(ts);

    let x;
    let y;
    if real_ratio <= target_ratio {
        let delta = (rs.x as f32 - rs.y as f32 / target_ratio) * 0.5;
        y = v.y / ratio.y;
        x = (v.x - delta) / ratio.y;
    } else {
        let delta = (rs.y as f32 - rs.x as f32 * target_ratio) * 0.5;
        x = v.x / ratio.x;
        y = (v.y - delta) / ratio.x;
    }

    Vec2i::new(x as _, y as _)
}

// Returns the mouse position relative to the actual window,
// without taking the target size into account (so if the window aspect ratio
// does not match with the target ratio, the result does not take "black bands" into account).
// Use this when you want to unproject mouse coordinates!
pub fn raw_mouse_pos_in_window<W: AsRef<Window_Handle>>(window: &W) -> Vec2i {
    backend::raw_mouse_pos_in_window(window.as_ref())
}

pub fn set_key_repeat_enabled<W: AsMut<Window_Handle>>(window: &mut W, enabled: bool) {
    backend::set_key_repeat_enabled(window.as_mut(), enabled);
}
