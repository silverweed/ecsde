use super::Mouse_Button;
use glfw::{Action, WindowEvent};
use inle_win::window::Event;

pub type Button = glfw::MouseButton;

const BUTTON_COUNT: usize = glfw::ffi::MOUSE_BUTTON_LAST as usize + 1;

pub(super) fn get_mouse_btn(btn: Button) -> Option<Mouse_Button> {
    match btn {
        // @Incomplete: verify these!
        Button::Button1 => Some(Mouse_Button::Left),
        Button::Button2 => Some(Mouse_Button::Right),
        Button::Button3 => Some(Mouse_Button::Middle),
        _ => None,
    }
}

const NUM_TO_MOUSE_BTN: [Button; BUTTON_COUNT] = [
    Button::Button1,
    Button::Button2,
    Button::Button3,
    Button::Button4,
    Button::Button5,
    Button::Button6,
    Button::Button7,
    Button::Button8,
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
            Mouse_Button::Left => Button::Button1,
            Mouse_Button::Right => Button::Button2,
            Mouse_Button::Middle => Button::Button3,
        }
    }
}

#[inline(always)]
pub const fn mousepressed(button: Button) -> Event {
    Event::Window(WindowEvent::MouseButton(
        button,
        Action::Press,
        glfw::Modifiers::empty(),
    ))
}

#[inline(always)]
pub const fn mousereleased(button: Button) -> Event {
    Event::Window(WindowEvent::MouseButton(
        button,
        Action::Release,
        glfw::Modifiers::empty(),
    ))
}

#[inline(always)]
pub const fn wheelscrolled(delta: f32) -> Event {
    Event::Window(WindowEvent::Scroll(0., delta as _))
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
