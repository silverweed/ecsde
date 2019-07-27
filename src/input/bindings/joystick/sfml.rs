use super::{Joystick, Joystick_Button, Joystick_Type};

pub(super) fn get_joy_btn_id(joystick: &Joystick, button: Joystick_Button) -> Option<u32> {
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
