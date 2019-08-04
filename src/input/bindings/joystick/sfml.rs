use super::Joystick_Axis;
use sfml::window::joystick::{self, Axis};

#[cfg(any(target_os = "linux", target_os = "windows"))]
pub(super) const AXES_ENGINE_TO_FRAMEWORK_XBOX360: [Option<Axis>; Joystick_Axis::_Count as usize] = [
    Some(Axis::X),    // Stick_Left_H
    Some(Axis::Y),    // Stick_Left_V,
    Some(Axis::U),    // Stick_Right_H,
    Some(Axis::V),    // Stick_Right_V,
    Some(Axis::Z),    // Trigger_Left,
    Some(Axis::R),    // Trigger_Right,
    Some(Axis::PovX), // Dpad_H,
    Some(Axis::PovY), // Dpad_V,
];

#[cfg(any(target_os = "linux", target_os = "windows"))]
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

pub(super) fn get_axis_value_xbox360(joystick_id: u32, axis: Joystick_Axis) -> f32 {
    assert!((axis as usize) < AXES_ENGINE_TO_FRAMEWORK_XBOX360.len());
    if let Some(ax) = AXES_ENGINE_TO_FRAMEWORK_XBOX360[axis as usize] {
        let (min, max) = AXES_RANGES_XBOX360[axis as usize];
        norm_minus_one_to_one(joystick::axis_position(joystick_id, ax), min, max)
    } else {
        0.0
    }
}

#[inline(always)]
fn norm_minus_one_to_one(x: f32, min: f32, max: f32) -> f32 {
    2.0 * (x - min) / (max - min) - 1.0
}
