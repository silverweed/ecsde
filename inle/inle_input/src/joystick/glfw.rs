use super::{Joystick_Axis, Joystick_Id};
use glfw::GamepadAxis;
use inle_win::window::Window_Handle;

pub(super) const JOY_COUNT: u32 = glfw::ffi::JOYSTICK_LAST as u32 + 1;

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub(super) const AXES_ENGINE_TO_FRAMEWORK_XBOX360: [Option<GamepadAxis>;
    Joystick_Axis::_Count as usize] = [
    Some(GamepadAxis::AxisLeftX),        // Stick_Left_H
    Some(GamepadAxis::AxisLeftY),        // Stick_Left_V,
    Some(GamepadAxis::AxisRightX),       // Stick_Right_H,
    Some(GamepadAxis::AxisRightY),       // Stick_Right_V,
    Some(GamepadAxis::AxisLeftTrigger),  // Trigger_Left,
    Some(GamepadAxis::AxisRightTrigger), // Trigger_Right,
    None,
    None,
];

// All these down here are @Incomplete! Right now they're copypasted by SFML

#[cfg(target_os = "linux")]
pub(super) const AXES_RANGES_XBOX360: [(f32, f32); Joystick_Axis::_Count as usize] = [
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
];

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub(super) const AXES_RANGES_XBOX360: [(f32, f32); Joystick_Axis::_Count as usize] = [
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (1.0, -1.0), // The Dpad_V is inverted on Windows and OSX
];

pub(super) fn get_joy_axis_value_xbox360(
    window: &Window_Handle,
    joystick_id: u32,
    axis: Joystick_Axis,
) -> f32 {
    assert!((axis as usize) < AXES_ENGINE_TO_FRAMEWORK_XBOX360.len());
    if let Some(ax) = AXES_ENGINE_TO_FRAMEWORK_XBOX360[axis as usize] {
        let (min, max) = AXES_RANGES_XBOX360[axis as usize];
        let joy = glfw::Joystick {
            glfw: window.glfw.clone(),
            id: engine_to_framework_joy_id(joystick_id),
        };
        if let Some(gamepad_state) = joy.get_gamepad_state() {
            let axis_pos = gamepad_state.get_axis(ax);
            norm_minus_one_to_one(axis_pos, min, max)
        } else {
            0.0
        }
    } else {
        0.0
    }
}

#[inline]
pub(super) fn is_joy_connected_internal(
    window: &Window_Handle,
    id: Joystick_Id,
) -> (bool, Option<Box<str>>) {
    let joy = glfw::Joystick {
        id: engine_to_framework_joy_id(id),
        glfw: window.glfw.clone(),
    };

    (joy.is_present(), joy.get_guid().map(|s| s.into_boxed_str()))
}

fn engine_to_framework_joy_id(id: Joystick_Id) -> glfw::JoystickId {
    debug_assert!(id < glfw::ffi::JOYSTICK_LAST as u32);
    // Note: this is safe because glfw::JoystickId is repr(i32)
    unsafe { std::mem::transmute(id) }
}

#[inline(always)]
fn norm_minus_one_to_one(x: f32, min: f32, max: f32) -> f32 {
    2.0 * (x - min) / (max - min) - 1.0
}

#[inline(always)]
pub(super) fn update_joysticks() {}
