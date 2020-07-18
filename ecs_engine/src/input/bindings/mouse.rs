use std::convert::TryFrom;

#[cfg(feature = "win-sfml")]
pub mod sfml;

#[cfg(feature = "win-glfw")]
pub mod glfw;

#[cfg(feature = "win-sfml")]
use self::sfml as backend;

#[cfg(feature = "win-glfw")]
use self::glfw as backend;

pub type Button = backend::Button;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(u8)]
pub enum Mouse_Button {
    Left,
    Right,
    Middle,
}

impl TryFrom<u8> for Mouse_Button {
    type Error = String;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        use Mouse_Button::*;
        match v {
            0 => Ok(Left),
            1 => Ok(Right),
            2 => Ok(Middle),
            _ => Err(format!("Invalid Mouse_Button: {}", v)),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Mouse_State {
    // indexed by Mouse_Button
    was_pressed_latest_frame: [bool; 3],
    is_pressed: [bool; 3],
}

pub fn update_mouse_state(state: &mut Mouse_State) {
    for i in 0..state.was_pressed_latest_frame.len() {
        state.was_pressed_latest_frame[i] = state.is_pressed[i];
        state.is_pressed[i] =
            backend::is_mouse_btn_pressed(Mouse_Button::try_from(i as u8).unwrap());
    }
}

pub fn mouse_went_up(state: &Mouse_State, button: Mouse_Button) -> bool {
    state.was_pressed_latest_frame[button as usize] && !is_mouse_btn_pressed(state, button)
}

pub fn mouse_went_down(state: &Mouse_State, button: Mouse_Button) -> bool {
    !state.was_pressed_latest_frame[button as usize] && is_mouse_btn_pressed(state, button)
}

pub fn string_to_mouse_btn(s: &str) -> Option<Mouse_Button> {
    match s {
        "Left" => Some(Mouse_Button::Left),
        "Right" => Some(Mouse_Button::Right),
        "Middle" => Some(Mouse_Button::Middle),
        _ => None,
    }
}

pub fn num_to_mouse_btn(num: usize) -> Option<Mouse_Button> {
    backend::num_to_mouse_btn(num)
}

pub fn get_mouse_btn(button: backend::Button) -> Option<Mouse_Button> {
    backend::get_mouse_btn(button)
}

#[inline(always)]
pub fn is_mouse_btn_pressed(state: &Mouse_State, button: Mouse_Button) -> bool {
    state.is_pressed[button as usize]
}
