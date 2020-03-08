#[cfg(feature = "use-sfml")]
pub mod sfml;

#[cfg(feature = "use-sfml")]
use self::sfml as backend;

pub type Button = backend::Button;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Mouse_Button {
    Left,
    Right,
    Middle,
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

pub fn is_mouse_btn_pressed(button: Mouse_Button) -> bool {
    backend::is_mouse_btn_pressed(button)
}
