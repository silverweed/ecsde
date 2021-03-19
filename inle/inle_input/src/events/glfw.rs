use crate::joystick::Joystick_Id;
use crate::keyboard;
use crate::mouse;
//use crate::input::joystick::{Joystick_Id, Button_Id as Joystick_Button_Id};
use super::Input_Raw_Event;
use inle_win::window::Event as Win_Event;

pub(super) fn framework_to_engine_event(event: Win_Event) -> Option<Input_Raw_Event> {
    use glfw::{Action, WindowEvent};

    match event {
        Win_Event::Window(WindowEvent::Close) => Some(Input_Raw_Event::Quit),
        Win_Event::Window(WindowEvent::FramebufferSize(width, height)) => Some(
            Input_Raw_Event::Resized(width.max(1) as u32, height.max(1) as u32),
        ),
        Win_Event::Window(WindowEvent::MouseButton(btn, Action::Press, _)) => {
            if let Some(button) = mouse::get_mouse_btn(btn) {
                Some(Input_Raw_Event::Mouse_Button_Pressed { button })
            } else {
                ldebug!("Ignored unknown button {:?}", btn);
                None
            }
        }
        Win_Event::Window(WindowEvent::MouseButton(btn, Action::Release, _)) => {
            if let Some(button) = mouse::get_mouse_btn(btn) {
                Some(Input_Raw_Event::Mouse_Button_Released { button })
            } else {
                ldebug!("Ignored unknown button {:?}", btn);
                None
            }
        }
        Win_Event::Window(WindowEvent::CursorPos(x, y)) => {
            Some(Input_Raw_Event::Mouse_Moved { x, y })
        }
        Win_Event::Window(WindowEvent::Scroll(_x, y)) => {
            Some(Input_Raw_Event::Mouse_Wheel_Scrolled {
                delta: y as f32, // @Robustness: truncating! Also, do we want to support horizontal scrolling?
            })
        }
        Win_Event::Window(WindowEvent::Key(key, _, Action::Press, _)) => {
            if let Some(key) = keyboard::framework_to_engine_key(key) {
                Some(Input_Raw_Event::Key_Pressed { code: key })
            } else {
                ldebug!("Ignored unknown key {:?}", key);
                None
            }
        }
        Win_Event::Window(WindowEvent::Key(key, _, Action::Release, _)) => {
            if let Some(key) = keyboard::framework_to_engine_key(key) {
                Some(Input_Raw_Event::Key_Released { code: key })
            } else {
                ldebug!("Ignored unknown key {:?}", key);
                None
            }
        }
        Win_Event::Window(WindowEvent::Key(key, _, Action::Repeat, _)) => {
            if let Some(key) = keyboard::framework_to_engine_key(key) {
                Some(Input_Raw_Event::Key_Repeated { code: key })
            } else {
                ldebug!("Ignored unknown key {:?}", key);
                None
            }
        }
        Win_Event::Joystick {
            joy_id,
            connected,
            guid,
        } => {
            if connected {
                Some(Input_Raw_Event::Joy_Connected {
                    id: joy_id as Joystick_Id,
                    guid,
                })
            } else {
                Some(Input_Raw_Event::Joy_Disconnected {
                    id: joy_id as Joystick_Id,
                    guid,
                })
            }
        }
        _ => None,
    }
}
