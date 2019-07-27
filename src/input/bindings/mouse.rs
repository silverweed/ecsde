#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
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
