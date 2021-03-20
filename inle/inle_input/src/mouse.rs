use crate::events::Input_Raw_Event;
use inle_math::vector::{Vec2f, Vec2i};
use inle_win::window::Window_Handle;
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
    cursor: (f64, f64),
}

pub fn reset_mouse_state(state: &mut Mouse_State) {
    // @WaitForStable: use const_assert if const len() ever becomes a thing
    debug_assert!(state.is_pressed.len() == state.was_pressed_latest_frame.len());
    for i in 0..state.is_pressed.len() {
        state.was_pressed_latest_frame[i] = false;
        state.is_pressed[i] = false;
    }
}

pub fn update_mouse_state(state: &mut Mouse_State, events: &[Input_Raw_Event]) {
    for i in 0..state.was_pressed_latest_frame.len() {
        state.was_pressed_latest_frame[i] = state.is_pressed[i];
    }

    for evt in events {
        match *evt {
            Input_Raw_Event::Mouse_Button_Pressed { button } => {
                state.is_pressed[button as usize] = true;
            }
            Input_Raw_Event::Mouse_Button_Released { button } => {
                state.is_pressed[button as usize] = false;
            }
            Input_Raw_Event::Mouse_Moved { x, y } => {
                state.cursor = (x, y);
            }
            _ => (),
        }
    }
}

#[inline]
pub fn mouse_went_up(state: &Mouse_State, button: Mouse_Button) -> bool {
    state.was_pressed_latest_frame[button as usize] && !is_mouse_btn_pressed(state, button)
}

#[inline]
pub fn mouse_went_down(state: &Mouse_State, button: Mouse_Button) -> bool {
    !state.was_pressed_latest_frame[button as usize] && is_mouse_btn_pressed(state, button)
}

/// Returns the mouse position relative to the window,
/// without taking the target size into account (so if the window aspect ratio
/// does not match with the target ratio, the result does not take "black bands" into account).
/// Use this when you want to unproject mouse coordinates!
#[inline(always)]
pub fn raw_mouse_pos(state: &Mouse_State) -> (f32, f32) {
    (state.cursor.0 as f32, state.cursor.1 as f32)
}

#[inline]
pub fn mouse_pos_in_window<W: AsRef<Window_Handle>>(window: &W, state: &Mouse_State) -> Vec2i {
    let raw = Vec2f::new(state.cursor.0 as f32, state.cursor.1 as f32);
    inle_win::window::correct_mouse_pos_in_window(window, raw)
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
