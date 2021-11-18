use inle_math::rect::{Rect, Rectf};
use inle_math::transform::Transform2D;
use inle_math::vector::{Vec2f, Vec2i, Vec2u};

#[cfg(feature = "win-sfml")]
mod sfml;

#[cfg(feature = "win-glfw")]
mod glfw;

#[cfg(feature = "win-winit")]
mod winit;

#[cfg(feature = "win-sfml")]
use self::sfml as backend;

#[cfg(feature = "win-glfw")]
use self::glfw as backend;

#[cfg(feature = "win-winit")]
use self::winit as backend;

#[cfg(all(feature = "win-glfw", feature = "gfx-gl"))]
pub use backend::get_gl_handle;

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

pub fn recreate_window<W: AsMut<Window_Handle>>(window: &mut W) {
    backend::recreate_window(window.as_mut());
}

#[inline(always)]
pub fn has_vsync<W: AsRef<Window_Handle>>(window: &W) -> bool {
    backend::has_vsync(window.as_ref())
}

#[inline(always)]
pub fn set_vsync<W: AsMut<Window_Handle>>(window: &mut W, vsync: bool) {
    backend::set_vsync(window.as_mut(), vsync);
}

#[inline(always)]
pub fn display<W: AsMut<Window_Handle>>(window: &mut W) {
    backend::display(window.as_mut());
}

#[inline(always)]
pub fn get_window_target_size<W: AsRef<Window_Handle>>(window: &W) -> (u32, u32) {
    trace!("get_window_target_size");
    backend::get_window_target_size(window.as_ref())
}

#[inline(always)]
pub fn get_window_real_size<W: AsRef<Window_Handle>>(window: &W) -> (u32, u32) {
    trace!("get_window_real_size");
    backend::get_window_real_size(window.as_ref())
}

#[inline]
pub fn get_camera_viewport<W: AsRef<Window_Handle>>(window: &W, camera: &Transform2D) -> Rectf {
    let viewport = Vec2f::from(Vec2u::from(get_window_target_size(window)));
    let mut visible = Rect::from_topleft_size(camera.position(), viewport * camera.scale());
    visible = visible - visible.size() * 0.5;
    visible
}

#[inline(always)]
pub fn prepare_poll_events<W: AsMut<Window_Handle>>(window: &mut W) {
    backend::prepare_poll_events(window.as_mut())
}

#[inline(always)]
pub fn poll_event<W: AsMut<Window_Handle>>(window: &mut W) -> Option<Event> {
    backend::poll_event(window.as_mut())
}

/// Converts a raw pos (one in pixels) to a scaled pos (one in screen coordinates)
/// Should be passed the position taken from `mouse::raw_mouse_pos`.
pub fn correct_mouse_pos_in_window<W: AsRef<Window_Handle>>(window: &W, raw_pos: Vec2f) -> Vec2i {
    trace!("correct_mouse_pos_in_window");

    let ts: Vec2u = get_window_target_size(window).into();
    let target_ratio = ts.y as f32 / ts.x as f32;
    debug_assert!(!target_ratio.is_nan());

    let rs: Vec2u = get_window_real_size(window).into();
    if rs.x == 0 {
        // This can happen if the window is minimized.
        return Vec2i::default();
    }

    let real_ratio = rs.y as f32 / rs.x as f32;
    debug_assert!(!real_ratio.is_nan());

    let ratio = Vec2f::from(rs) / Vec2f::from(ts);
    inle_math::vector::sanity_check_v(ratio);

    let x;
    let y;
    if real_ratio <= target_ratio {
        let delta = (rs.x as f32 - rs.y as f32 / target_ratio) * 0.5;
        y = raw_pos.y / ratio.y;
        x = (raw_pos.x - delta) / ratio.y;
    } else {
        let delta = (rs.y as f32 - rs.x as f32 * target_ratio) * 0.5;
        x = raw_pos.x / ratio.x;
        y = (raw_pos.y - delta) / ratio.x;
    }

    Vec2i::new(x as _, y as _)
}

#[inline(always)]
pub fn set_key_repeat_enabled<W: AsMut<Window_Handle>>(window: &mut W, enabled: bool) {
    backend::set_key_repeat_enabled(window.as_mut(), enabled);
}
