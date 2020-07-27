use super::{Joystick_Axis, Joystick_Type};
use glfw::GamepadAxis;

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
    (-100.0, 100.0),
    (-100.0, 100.0),
    (-100.0, 100.0),
    (-100.0, 100.0),
    (-100.0, 100.0),
    (-100.0, 100.0),
    (-100.0, 100.0),
    (-100.0, 100.0),
];

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub(super) const AXES_RANGES_XBOX360: [(f32, f32); Joystick_Axis::_Count as usize] = [
    (-100.0, 100.0),
    (-100.0, 100.0),
    (-100.0, 100.0),
    (-100.0, 100.0),
    (-100.0, 100.0),
    (-100.0, 100.0),
    (-100.0, 100.0),
    (100.0, -100.0), // The Dpad_V is inverted on Windows and OSX
];

pub(super) fn get_axis_value_xbox360(joystick_id: u32, axis: Joystick_Axis) -> f32 {
    assert!((axis as usize) < AXES_ENGINE_TO_FRAMEWORK_XBOX360.len());
    if let Some(ax) = AXES_ENGINE_TO_FRAMEWORK_XBOX360[axis as usize] {
        let (min, max) = AXES_RANGES_XBOX360[axis as usize];
        let axis_pos = 0.; // @Incomplete!
        norm_minus_one_to_one(axis_pos, min, max)
    } else {
        0.0
    }
}

#[inline(always)]
pub(super) fn is_connected(joystick_id: u32) -> bool {
    false
}

pub fn get_joy_type(id: u32) -> Result<Joystick_Type, &'static str> {
    if !is_connected(id) {
        return Err("Joystick is not connected.");
    }

    // @Incomplete @Temporary: for now we only support XBox360
    Ok(Joystick_Type::XBox360)
}

#[inline(always)]
fn norm_minus_one_to_one(x: f32, min: f32, max: f32) -> f32 {
    2.0 * (x - min) / (max - min) - 1.0
}

pub(super) fn get_connected_joysticks_mask() -> super::Joystick_Mask {
    let mut mask = 0u8;
    // @Incomplete
    //for i in 0..joystick::COUNT {
    //mask |= (is_connected(i) as u8) << i;
    //}
    mask
}

#[inline(always)]
pub(super) fn update_joysticks() {}
