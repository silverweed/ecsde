use super::Joystick_Axis;
use sfml::window::joystick::Axis;

pub(super) type Framework_Joy_Axis = sfml::window::joystick::Axis;

#[cfg(target_os = "linux")]
pub(super) const AXES_TO_IDS_XBOX360: [Option<u32>; Joystick_Axis::_Count as usize] = [
    Some(Axis::X as u32),    // Stick_Left_H
    Some(Axis::Y as u32),    // Stick_Left_V,
    Some(Axis::U as u32),    // Stick_Right_H,
    Some(Axis::V as u32),    // Stick_Right_V,
    Some(Axis::Z as u32),    // Trigger_Left,
    Some(Axis::R as u32),    // Trigger_Right,
    Some(Axis::PovX as u32), // Dpad_H,
    Some(Axis::PovY as u32), // Dpad_V,
];

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
