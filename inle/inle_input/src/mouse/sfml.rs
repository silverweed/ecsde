use super::Mouse_Button;
pub(super) use sfml::window::mouse::{Button, Wheel};
use sfml::window::Event;

pub(super) fn get_mouse_btn(btn: Button) -> Option<Mouse_Button> {
    match btn {
        Button::Left => Some(Mouse_Button::Left),
        Button::Right => Some(Mouse_Button::Right),
        Button::Middle => Some(Mouse_Button::Middle),
        _ => None,
    }
}

const NUM_TO_MOUSE_BTN: [Button; Button::Count as usize] = [
    Button::Left,
    Button::Right,
    Button::Middle,
    Button::XButton1,
    Button::XButton2,
];

pub(super) fn num_to_mouse_btn(num: usize) -> Option<Mouse_Button> {
    if num < NUM_TO_MOUSE_BTN.len() {
        get_mouse_btn(NUM_TO_MOUSE_BTN[num])
    } else {
        None
    }
}

impl std::convert::From<Mouse_Button> for Button {
    fn from(b: Mouse_Button) -> Button {
        match b {
            Mouse_Button::Left => Button::Left,
            Mouse_Button::Right => Button::Right,
            Mouse_Button::Middle => Button::Middle,
        }
    }
}

#[inline(always)]
pub const fn mousepressed(button: Button) -> Event {
    Event::MouseButtonPressed { button, x: 0, y: 0 }
}

#[inline(always)]
pub const fn mousereleased(button: Button) -> Event {
    Event::MouseButtonReleased { button, x: 0, y: 0 }
}

#[inline(always)]
pub const fn wheelscrolled(delta: f32) -> Event {
    Event::MouseWheelScrolled {
        wheel: Wheel::Vertical,
        delta,
        x: 0,
        y: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_to_mouse_btn() {
        assert_eq!(
            num_to_mouse_btn(Mouse_Button::Left as usize),
            Some(Mouse_Button::Left)
        );
        assert_eq!(
            num_to_mouse_btn(Mouse_Button::Right as usize),
            Some(Mouse_Button::Right)
        );
        assert_eq!(
            num_to_mouse_btn(Mouse_Button::Middle as usize),
            Some(Mouse_Button::Middle)
        );
    }
}
