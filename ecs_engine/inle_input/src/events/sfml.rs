use crate::keyboard;
use crate::mouse;
//use crate::input::joystick::{Joystick_Id, Button_Id as Joystick_Button_Id};
use super::Input_Raw_Event;
use inle_win::window::Event as Win_Event;

pub(super) fn framework_to_engine_event(event: Win_Event) -> Option<Input_Raw_Event> {
    match event {
        Win_Event::Closed => Some(Input_Raw_Event::Quit),
        Win_Event::Resized { width, height } => Some(Input_Raw_Event::Resized(
            width.max(1) as u32,
            height.max(1) as u32,
        )),
        Win_Event::GainedFocus => Some(Input_Raw_Event::Focus_Gained),
        Win_Event::LostFocus => Some(Input_Raw_Event::Focus_Lost),
        Win_Event::MouseButtonPressed { button, .. } => {
            if let Some(button) = mouse::get_mouse_btn(button) {
                Some(Input_Raw_Event::Mouse_Button_Pressed { button })
            } else {
                ldebug!("Ignored unknown button {:?}", button);
                None
            }
        }
        Win_Event::MouseButtonReleased { button, .. } => {
            if let Some(button) = mouse::get_mouse_btn(button) {
                Some(Input_Raw_Event::Mouse_Button_Released { button })
            } else {
                ldebug!("Ignored unknown button {:?}", button);
                None
            }
        }
        Win_Event::MouseWheelScrolled { delta, .. } => {
            Some(Input_Raw_Event::Mouse_Wheel_Scrolled { delta })
        }
        Win_Event::KeyPressed { code: key, .. } => {
            if let Some(key) = keyboard::framework_to_engine_key(key) {
                Some(Input_Raw_Event::Key_Pressed { code: key })
            } else {
                ldebug!("Ignored unknown key {:?}", key);
                None
            }
        }
        Win_Event::KeyReleased { code: key, .. } => {
            if let Some(key) = keyboard::framework_to_engine_key(key) {
                Some(Input_Raw_Event::Key_Released { code: key })
            } else {
                ldebug!("Ignored unknown key {:?}", key);
                None
            }
        }
        Win_Event::JoystickButtonPressed {
            joystickid, button, ..
        } => Some(Input_Raw_Event::Joy_Button_Pressed {
            joystick_id: joystickid,
            button,
        }),
        Win_Event::JoystickButtonReleased {
            joystickid, button, ..
        } => Some(Input_Raw_Event::Joy_Button_Released {
            joystick_id: joystickid,
            button,
        }),
        _ => None,
    }
}
