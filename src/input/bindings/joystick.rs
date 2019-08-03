#[cfg(feature = "use-sfml")]
mod sfml;

#[derive(Copy, Clone, Debug)]
pub enum Joystick_Type {
    XBox360,
}

#[derive(Copy, Clone, Debug)]
pub struct Joystick {
    pub id: u32,
    pub joy_type: Joystick_Type,
}

// Don't change the order of these!
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Joystick_Button {
    /// Triangle
    Face_Top,
    /// Circle
    Face_Right,
    /// Square
    Face_Bottom,
    /// Cross
    Face_Left,
    /// Select
    Special_Left,
    /// Start
    Special_Right,
    /// XBox Button
    Special_Middle,
    /// L3
    Stick_Left,
    /// R3
    Stick_Right,
    /// L1
    Shoulder_Left,
    /// R1
    Shoulder_Right,
    Dpad_Top,
    Dpad_Right,
    Dpad_Bottom,
    Dpad_Left,
    // L2
    Trigger_Left,
    // R2
    Trigger_Right,
    _Count,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Joystick_Axis {
    Stick_Left_H,
    Stick_Left_V,
    Stick_Right_H,
    Stick_Right_V,
    Trigger_Left,
    Trigger_Right,
    Dpad_H,
    Dpad_V,
    _Count,
}

pub fn string_to_joy_btn(s: &str) -> Option<Joystick_Button> {
    match s {
        "Face_Top" => Some(Joystick_Button::Face_Top),
        "Face_Right" => Some(Joystick_Button::Face_Right),
        "Face_Bottom" => Some(Joystick_Button::Face_Bottom),
        "Face_Left" => Some(Joystick_Button::Face_Left),
        "Special_Left" => Some(Joystick_Button::Special_Left),
        "Special_Right" => Some(Joystick_Button::Special_Right),
        "Special_Middle" => Some(Joystick_Button::Special_Middle),
        "Stick_Left" => Some(Joystick_Button::Stick_Left),
        "Stick_Right" => Some(Joystick_Button::Stick_Right),
        "Shoulder_Left" => Some(Joystick_Button::Shoulder_Left),
        "Shoulder_Right" => Some(Joystick_Button::Shoulder_Right),
        "Dpad_Top" => Some(Joystick_Button::Dpad_Top),
        "Dpad_Right" => Some(Joystick_Button::Dpad_Right),
        "Dpad_Bottom" => Some(Joystick_Button::Dpad_Bottom),
        "Dpad_Left" => Some(Joystick_Button::Dpad_Left),
        "Trigger_Left" => Some(Joystick_Button::Trigger_Left),
        "Trigger_Right" => Some(Joystick_Button::Trigger_Right),
        _ => None,
    }
}

pub fn string_to_joy_axis(s: &str) -> Option<Joystick_Axis> {
    match s {
        "Stick_Left_H" => Some(Joystick_Axis::Stick_Left_H),
        "Stick_Left_V" => Some(Joystick_Axis::Stick_Left_V),
        "Stick_Right_H" => Some(Joystick_Axis::Stick_Right_H),
        "Stick_Right_V" => Some(Joystick_Axis::Stick_Right_V),
        "Trigger_Left" => Some(Joystick_Axis::Trigger_Left),
        "Trigger_Right" => Some(Joystick_Axis::Trigger_Right),
        "Dpad_H" => Some(Joystick_Axis::Dpad_H),
        "Dpad_V" => Some(Joystick_Axis::Dpad_V),
        _ => None,
    }
}

pub fn get_joy_btn_id(joystick: Joystick, button: Joystick_Button) -> Option<u32> {
    match joystick.joy_type {
        Joystick_Type::XBox360 => get_joy_btn_id_xbox360(button),
    }
}

#[inline]
fn get_joy_btn_id_xbox360(button: Joystick_Button) -> Option<u32> {
    assert!((button as usize) < BUTTONS_TO_IDS_XBOX360.len());
    BUTTONS_TO_IDS_XBOX360[button as usize]
}

pub fn get_joy_btn_from_id(joystick: Joystick, id: u32) -> Option<Joystick_Button> {
    match joystick.joy_type {
        Joystick_Type::XBox360 => get_joy_btn_from_id_xbox360(id),
    }
}

#[inline]
fn get_joy_btn_from_id_xbox360(id: u32) -> Option<Joystick_Button> {
    if (id as usize) < IDS_TO_BUTTONS_XBOX360.len() {
        Some(IDS_TO_BUTTONS_XBOX360[id as usize])
    } else {
        None
    }
}

pub fn get_joy_axis_id(joystick: Joystick, axis: Joystick_Axis) -> Option<u32> {
    match joystick.joy_type {
        Joystick_Type::XBox360 => get_joy_axis_id_xbox360(axis),
    }
}

#[inline]
pub fn get_joy_axis_id_xbox360(axis: Joystick_Axis) -> Option<u32> {
    assert!((axis as usize) < AXES_TO_IDS_XBOX360.len());
    AXES_TO_IDS_XBOX360[axis as usize]
}

// Map (Joystick_Button as u8) => (button id)
#[cfg(target_os = "linux")]
const BUTTONS_TO_IDS_XBOX360: [Option<u32>; Joystick_Button::_Count as usize] = [
    Some(3),  // Face_Top
    Some(1),  // Face_Right
    Some(0),  // Face_Bottom
    Some(2),  // Face_Left
    Some(6),  // Special_Left
    Some(7),  // Special_Right
    Some(8),  // Special_Middle
    Some(9),  // Stick_Left
    Some(10), // Stick_Right
    Some(4),  // Shoulder_Left
    Some(5),  // Shoulder_Right
    None,     // Dpad_Top
    None,     // Dpad_Right
    None,     // Dpad_Bottom
    None,     // Dpad_Left
    None,     // Trigger_Left
    None,     // Trigger_Right
];

#[cfg(target_os = "windows")]
const BUTTONS_TO_IDS_XBOX360: [Option<u32>; Joystick_Button::_Count as usize] = [
    Some(3),  // Face_Top
    Some(1),  // Face_Right
    Some(0),  // Face_Bottom
    Some(2),  // Face_Left
    Some(6),  // Special_Left
    Some(7),  // Special_Right
    None,     // Special_Middle
    Some(9),  // Stick_Left
    Some(10), // Stick_Right
    Some(4),  // Shoulder_Left
    Some(5),  // Shoulder_Right
    None,     // Dpad_Top
    None,     // Dpad_Right
    None,     // Dpad_Bottom
    None,     // Dpad_Left
    None,     // Trigger_Left
    None,     // Trigger_Right
];

#[cfg(target_os = "osx")]
const BUTTONS_TO_IDS_XBOX360: [Option<u32>; Joystick_Button::_Count as usize] = [
    Some(3),  // Face_Top
    Some(2),  // Face_Right
    Some(1),  // Face_Bottom
    Some(0),  // Face_Left
    Some(8),  // Special_Left
    Some(9),  // Special_Right
    Some(12), // Special_Middle
    Some(10), // Stick_Left
    Some(11), // Stick_Right
    Some(4),  // Shoulder_Left
    Some(5),  // Shoulder_Right
    None,     // Dpad_Top
    None,     // Dpad_Right
    None,     // Dpad_Bottom
    None,     // Dpad_Left
    Some(6),  // Trigger_Left
    Some(7),  // Trigger_Right
];

// @Incomplete: button mapping on OSX may not range from 0 to 11: in that case we'll probably need
// a hash map or something...
#[cfg(target_os = "linux")]
const IDS_TO_BUTTONS_XBOX360: [Joystick_Button; 11] = [
    Joystick_Button::Face_Bottom,
    Joystick_Button::Face_Right,
    Joystick_Button::Face_Left,
    Joystick_Button::Face_Top,
    Joystick_Button::Shoulder_Left,
    Joystick_Button::Shoulder_Right,
    Joystick_Button::Special_Left,
    Joystick_Button::Special_Right,
    Joystick_Button::Special_Middle,
    Joystick_Button::Stick_Left,
    Joystick_Button::Stick_Right,
];

#[cfg(target_os = "windows")]
const IDS_TO_BUTTONS_XBOX360: [Joystick_Button; 10] = [
    Joystick_Button::Face_Bottom,
    Joystick_Button::Face_Right,
    Joystick_Button::Face_Left,
    Joystick_Button::Face_Top,
    Joystick_Button::Shoulder_Left,
    Joystick_Button::Shoulder_Right,
    Joystick_Button::Special_Left,
    Joystick_Button::Special_Right,
    Joystick_Button::Stick_Left,
    Joystick_Button::Stick_Right,
];

#[cfg(target_os = "osx")]
const IDS_TO_BUTTONS_XBOX360: [Joystick_Button; 13] = [
    Joystick_Button::Face_Left,
    Joystick_Button::Face_Bottom,
    Joystick_Button::Face_Right,
    Joystick_Button::Face_Top,
    Joystick_Button::Shoulder_Left,
    Joystick_Button::Shoulder_Right,
    Joystick_Button::Trigger_Left,
    Joystick_Button::Trigger_Right,
    Joystick_Button::Special_Left,
    Joystick_Button::Special_Right,
    Joystick_Button::Stick_Left,
    Joystick_Button::Stick_Right,
    Joystick_Button::Special_Middle,
];

#[cfg(feature = "use-sfml")]
type Framework_Joy_Axis = sfml::Framework_Joy_Axis;

#[cfg(target_os = "linux")]
const AXES_TO_IDS_XBOX360: [Option<u32>; Joystick_Axis::_Count as usize] =
    sfml::AXES_TO_IDS_XBOX360;
