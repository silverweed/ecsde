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
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
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
        _ => None,
    }
}

pub fn get_joy_btn_id(joystick: &Joystick, button: Joystick_Button) -> Option<u32> {
    match joystick.joy_type {
        Joystick_Type::XBox360 => get_joy_btn_id_xbox360(button),
    }
}

// Map (Joystick_Button as u8) => (button id)
const BUTTONS_XBOX360: [Option<u32>; Joystick_Button::_Count as usize] = [
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
];

#[inline]
fn get_joy_btn_id_xbox360(button: Joystick_Button) -> Option<u32> {
    assert!((button as usize) < BUTTONS_XBOX360.len());
    BUTTONS_XBOX360[button as usize]
}
