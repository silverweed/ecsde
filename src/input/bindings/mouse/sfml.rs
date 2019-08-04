use super::Mouse_Button;
pub(super) use sfml::window::mouse::Button;

pub(super) fn get_mouse_btn(btn: Button) -> Option<Mouse_Button> {
    match btn {
        Button::Left => Some(Mouse_Button::Left),
        Button::Right => Some(Mouse_Button::Right),
        Button::Middle => Some(Mouse_Button::Middle),
        _ => None,
    }
}
