use crate::joystick::{Button_Id as Joystick_Button_Id, Joystick_Id};
use crate::keyboard::Key;
use crate::mouse::Mouse_Button;
use inle_win::window::Event as Win_Event;
use std::convert::TryFrom;

#[cfg(feature = "win-glfw")]
mod glfw;

#[cfg(feature = "win-glfw")]
use self::glfw as backend;

#[cfg(feature = "win-sfml")]
mod sfml;

#[cfg(feature = "win-sfml")]
use self::sfml as backend;

// Abstraction layer for input events

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Input_Raw_Event {
    Quit,
    Resized(u32, u32),
    Focus_Gained,
    Focus_Lost,
    Key_Pressed {
        code: Key,
    },
    Key_Released {
        code: Key,
    },
    Key_Repeated {
        code: Key,
    },
    Mouse_Wheel_Scrolled {
        delta: f32,
    },
    Mouse_Button_Pressed {
        button: Mouse_Button,
    },
    Mouse_Button_Released {
        button: Mouse_Button,
    },
    Mouse_Moved {
        x: f64,
        y: f64,
    },
    Joy_Button_Pressed {
        joystick_id: Joystick_Id,
        button: Joystick_Button_Id,
    },
    Joy_Button_Released {
        joystick_id: Joystick_Id,
        button: Joystick_Button_Id,
    },
    Joy_Connected {
        id: Joystick_Id,
        guid: Option<Box<str>>,
    },
    Joy_Disconnected {
        id: Joystick_Id,
        guid: Option<Box<str>>,
    },
}

impl TryFrom<Win_Event> for Input_Raw_Event {
    type Error = ();

    fn try_from(evt: Win_Event) -> Result<Self, Self::Error> {
        framework_to_engine_event(evt).ok_or(())
    }
}

pub fn framework_to_engine_event(event: Win_Event) -> Option<Input_Raw_Event> {
    backend::framework_to_engine_event(event)
}
