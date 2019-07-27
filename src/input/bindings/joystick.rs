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
#[derive(Copy, Clone)]
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

#[cfg(feature = "use-sfml")]
pub fn get_joy_btn_id(joystick: &Joystick, button: Joystick_Button) -> Option<u32> {
    sfml::get_joy_btn_id(joystick, button)
}
