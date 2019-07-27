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

#[cfg(feature = "use-sfml")]
use sfml::window::mouse::Button as SfButton;

#[cfg(feature = "use-sfml")]
impl std::convert::TryFrom<SfButton> for Mouse_Button {
    type Error = &'static str;

    fn try_from(btn: SfButton) -> Result<Self, Self::Error> {
        match btn {
            SfButton::Left => Ok(Mouse_Button::Left),
            SfButton::Right => Ok(Mouse_Button::Right),
            SfButton::Middle => Ok(Mouse_Button::Middle),
            _ => Err("Invalid mouse button"),
        }
    }
}
