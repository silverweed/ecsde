use sfml::window::Event;
pub(super) use sfml::window::Key;

pub(super) fn string_to_key(s: &str) -> Option<Key> {
    match s {
        "A" => Some(Key::A),
        "B" => Some(Key::B),
        "C" => Some(Key::C),
        "D" => Some(Key::D),
        "E" => Some(Key::E),
        "F" => Some(Key::F),
        "G" => Some(Key::G),
        "H" => Some(Key::H),
        "I" => Some(Key::I),
        "J" => Some(Key::J),
        "K" => Some(Key::K),
        "L" => Some(Key::L),
        "M" => Some(Key::M),
        "N" => Some(Key::N),
        "O" => Some(Key::O),
        "P" => Some(Key::P),
        "Q" => Some(Key::Q),
        "R" => Some(Key::R),
        "S" => Some(Key::S),
        "T" => Some(Key::T),
        "U" => Some(Key::U),
        "V" => Some(Key::V),
        "W" => Some(Key::W),
        "X" => Some(Key::X),
        "Y" => Some(Key::Y),
        "Z" => Some(Key::Z),
        "Num0" => Some(Key::Num0),
        "Num1" => Some(Key::Num1),
        "Num2" => Some(Key::Num2),
        "Num3" => Some(Key::Num3),
        "Num4" => Some(Key::Num4),
        "Num5" => Some(Key::Num5),
        "Num6" => Some(Key::Num6),
        "Num7" => Some(Key::Num7),
        "Num8" => Some(Key::Num8),
        "Num9" => Some(Key::Num9),
        "Escape" => Some(Key::Escape),
        "LControl" => Some(Key::LControl),
        "LShift" => Some(Key::LShift),
        "LAlt" => Some(Key::LAlt),
        "LSystem" => Some(Key::LSystem),
        "RControl" => Some(Key::RControl),
        "RShift" => Some(Key::RShift),
        "RAlt" => Some(Key::RAlt),
        "RSystem" => Some(Key::RSystem),
        "Menu" => Some(Key::Menu),
        "LBracket" => Some(Key::LBracket),
        "RBracket" => Some(Key::RBracket),
        "SemiColon" => Some(Key::SemiColon),
        "Comma" => Some(Key::Comma),
        "Period" => Some(Key::Period),
        "Quote" => Some(Key::Quote),
        "Slash" => Some(Key::Slash),
        "BackSlash" => Some(Key::BackSlash),
        "Tilde" => Some(Key::Tilde),
        "Equal" => Some(Key::Equal),
        "Dash" => Some(Key::Dash),
        "Space" => Some(Key::Space),
        "Return" => Some(Key::Return),
        "BackSpace" => Some(Key::BackSpace),
        "Tab" => Some(Key::Tab),
        "PageUp" => Some(Key::PageUp),
        "PageDown" => Some(Key::PageDown),
        "End" => Some(Key::End),
        "Home" => Some(Key::Home),
        "Insert" => Some(Key::Insert),
        "Delete" => Some(Key::Delete),
        "Add" => Some(Key::Add),
        "Subtract" => Some(Key::Subtract),
        "Multiply" => Some(Key::Multiply),
        "Divide" => Some(Key::Divide),
        "Left" => Some(Key::Left),
        "Right" => Some(Key::Right),
        "Up" => Some(Key::Up),
        "Down" => Some(Key::Down),
        "Numpad0" => Some(Key::Numpad0),
        "Numpad1" => Some(Key::Numpad1),
        "Numpad2" => Some(Key::Numpad2),
        "Numpad3" => Some(Key::Numpad3),
        "Numpad4" => Some(Key::Numpad4),
        "Numpad5" => Some(Key::Numpad5),
        "Numpad6" => Some(Key::Numpad6),
        "Numpad7" => Some(Key::Numpad7),
        "Numpad8" => Some(Key::Numpad8),
        "Numpad9" => Some(Key::Numpad9),
        "F1" => Some(Key::F1),
        "F2" => Some(Key::F2),
        "F3" => Some(Key::F3),
        "F4" => Some(Key::F4),
        "F5" => Some(Key::F5),
        "F6" => Some(Key::F6),
        "F7" => Some(Key::F7),
        "F8" => Some(Key::F8),
        "F9" => Some(Key::F9),
        "F10" => Some(Key::F10),
        "F11" => Some(Key::F11),
        "F12" => Some(Key::F12),
        "F13" => Some(Key::F13),
        "F14" => Some(Key::F14),
        "F15" => Some(Key::F15),
        "Pause" => Some(Key::Pause),
        _ => None,
    }
}

const NUM_TO_KEY: [Key; 101] = [
    Key::A,
    Key::B,
    Key::C,
    Key::D,
    Key::E,
    Key::F,
    Key::G,
    Key::H,
    Key::I,
    Key::J,
    Key::K,
    Key::L,
    Key::M,
    Key::N,
    Key::O,
    Key::P,
    Key::Q,
    Key::R,
    Key::S,
    Key::T,
    Key::U,
    Key::V,
    Key::W,
    Key::X,
    Key::Y,
    Key::Z,
    Key::Num0,
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
    Key::Escape,
    Key::LControl,
    Key::LShift,
    Key::LAlt,
    Key::LSystem,
    Key::RControl,
    Key::RShift,
    Key::RAlt,
    Key::RSystem,
    Key::Menu,
    Key::LBracket,
    Key::RBracket,
    Key::SemiColon,
    Key::Comma,
    Key::Period,
    Key::Quote,
    Key::Slash,
    Key::BackSlash,
    Key::Tilde,
    Key::Equal,
    Key::Dash,
    Key::Space,
    Key::Return,
    Key::BackSpace,
    Key::Tab,
    Key::PageUp,
    Key::PageDown,
    Key::End,
    Key::Home,
    Key::Insert,
    Key::Delete,
    Key::Add,
    Key::Subtract,
    Key::Multiply,
    Key::Divide,
    Key::Left,
    Key::Right,
    Key::Up,
    Key::Down,
    Key::Numpad0,
    Key::Numpad1,
    Key::Numpad2,
    Key::Numpad3,
    Key::Numpad4,
    Key::Numpad5,
    Key::Numpad6,
    Key::Numpad7,
    Key::Numpad8,
    Key::Numpad9,
    Key::F1,
    Key::F2,
    Key::F3,
    Key::F4,
    Key::F5,
    Key::F6,
    Key::F7,
    Key::F8,
    Key::F9,
    Key::F10,
    Key::F11,
    Key::F12,
    Key::F13,
    Key::F14,
    Key::F15,
    Key::Pause,
];

pub(super) fn num_to_key(num: usize) -> Option<Key> {
    if num < NUM_TO_KEY.len() {
        Some(NUM_TO_KEY[num])
    } else {
        None
    }
}

#[inline(always)]
pub(super) fn is_key_pressed(key: Key) -> bool {
    key.is_pressed()
}

// @Cleanup @Temporary: make this a macro
#[inline(always)]
pub fn keypressed(code: Key) -> Event {
    Event::KeyPressed {
        code,
        alt: false,
        ctrl: false,
        shift: false,
        system: false,
    }
}

// @Cleanup @Temporary: make this a macro
#[inline(always)]
pub fn keyreleased(code: Key) -> Event {
    Event::KeyReleased {
        code,
        alt: false,
        ctrl: false,
        shift: false,
        system: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_to_key() {
        assert_eq!(num_to_key(Key::A as usize), Some(Key::A));
        assert_eq!(num_to_key(Key::B as usize), Some(Key::B));
        assert_eq!(num_to_key(Key::C as usize), Some(Key::C));
        assert_eq!(num_to_key(Key::D as usize), Some(Key::D));
        assert_eq!(num_to_key(Key::E as usize), Some(Key::E));
        assert_eq!(num_to_key(Key::F as usize), Some(Key::F));
        assert_eq!(num_to_key(Key::G as usize), Some(Key::G));
        assert_eq!(num_to_key(Key::H as usize), Some(Key::H));
        assert_eq!(num_to_key(Key::I as usize), Some(Key::I));
        assert_eq!(num_to_key(Key::J as usize), Some(Key::J));
        assert_eq!(num_to_key(Key::K as usize), Some(Key::K));
        assert_eq!(num_to_key(Key::L as usize), Some(Key::L));
        assert_eq!(num_to_key(Key::M as usize), Some(Key::M));
        assert_eq!(num_to_key(Key::N as usize), Some(Key::N));
        assert_eq!(num_to_key(Key::O as usize), Some(Key::O));
        assert_eq!(num_to_key(Key::P as usize), Some(Key::P));
        assert_eq!(num_to_key(Key::Q as usize), Some(Key::Q));
        assert_eq!(num_to_key(Key::R as usize), Some(Key::R));
        assert_eq!(num_to_key(Key::S as usize), Some(Key::S));
        assert_eq!(num_to_key(Key::T as usize), Some(Key::T));
        assert_eq!(num_to_key(Key::U as usize), Some(Key::U));
        assert_eq!(num_to_key(Key::V as usize), Some(Key::V));
        assert_eq!(num_to_key(Key::W as usize), Some(Key::W));
        assert_eq!(num_to_key(Key::X as usize), Some(Key::X));
        assert_eq!(num_to_key(Key::Y as usize), Some(Key::Y));
        assert_eq!(num_to_key(Key::Z as usize), Some(Key::Z));
        assert_eq!(num_to_key(Key::Num0 as usize), Some(Key::Num0));
        assert_eq!(num_to_key(Key::Num1 as usize), Some(Key::Num1));
        assert_eq!(num_to_key(Key::Num2 as usize), Some(Key::Num2));
        assert_eq!(num_to_key(Key::Num3 as usize), Some(Key::Num3));
        assert_eq!(num_to_key(Key::Num4 as usize), Some(Key::Num4));
        assert_eq!(num_to_key(Key::Num5 as usize), Some(Key::Num5));
        assert_eq!(num_to_key(Key::Num6 as usize), Some(Key::Num6));
        assert_eq!(num_to_key(Key::Num7 as usize), Some(Key::Num7));
        assert_eq!(num_to_key(Key::Num8 as usize), Some(Key::Num8));
        assert_eq!(num_to_key(Key::Num9 as usize), Some(Key::Num9));
        assert_eq!(num_to_key(Key::Escape as usize), Some(Key::Escape));
        assert_eq!(num_to_key(Key::LControl as usize), Some(Key::LControl));
        assert_eq!(num_to_key(Key::LShift as usize), Some(Key::LShift));
        assert_eq!(num_to_key(Key::LAlt as usize), Some(Key::LAlt));
        assert_eq!(num_to_key(Key::LSystem as usize), Some(Key::LSystem));
        assert_eq!(num_to_key(Key::RControl as usize), Some(Key::RControl));
        assert_eq!(num_to_key(Key::RShift as usize), Some(Key::RShift));
        assert_eq!(num_to_key(Key::RAlt as usize), Some(Key::RAlt));
        assert_eq!(num_to_key(Key::RSystem as usize), Some(Key::RSystem));
        assert_eq!(num_to_key(Key::Menu as usize), Some(Key::Menu));
        assert_eq!(num_to_key(Key::LBracket as usize), Some(Key::LBracket));
        assert_eq!(num_to_key(Key::RBracket as usize), Some(Key::RBracket));
        assert_eq!(num_to_key(Key::SemiColon as usize), Some(Key::SemiColon));
        assert_eq!(num_to_key(Key::Comma as usize), Some(Key::Comma));
        assert_eq!(num_to_key(Key::Period as usize), Some(Key::Period));
        assert_eq!(num_to_key(Key::Quote as usize), Some(Key::Quote));
        assert_eq!(num_to_key(Key::Slash as usize), Some(Key::Slash));
        assert_eq!(num_to_key(Key::BackSlash as usize), Some(Key::BackSlash));
        assert_eq!(num_to_key(Key::Tilde as usize), Some(Key::Tilde));
        assert_eq!(num_to_key(Key::Equal as usize), Some(Key::Equal));
        assert_eq!(num_to_key(Key::Dash as usize), Some(Key::Dash));
        assert_eq!(num_to_key(Key::Space as usize), Some(Key::Space));
        assert_eq!(num_to_key(Key::Return as usize), Some(Key::Return));
        assert_eq!(num_to_key(Key::BackSpace as usize), Some(Key::BackSpace));
        assert_eq!(num_to_key(Key::Tab as usize), Some(Key::Tab));
        assert_eq!(num_to_key(Key::PageUp as usize), Some(Key::PageUp));
        assert_eq!(num_to_key(Key::PageDown as usize), Some(Key::PageDown));
        assert_eq!(num_to_key(Key::End as usize), Some(Key::End));
        assert_eq!(num_to_key(Key::Home as usize), Some(Key::Home));
        assert_eq!(num_to_key(Key::Insert as usize), Some(Key::Insert));
        assert_eq!(num_to_key(Key::Delete as usize), Some(Key::Delete));
        assert_eq!(num_to_key(Key::Add as usize), Some(Key::Add));
        assert_eq!(num_to_key(Key::Subtract as usize), Some(Key::Subtract));
        assert_eq!(num_to_key(Key::Multiply as usize), Some(Key::Multiply));
        assert_eq!(num_to_key(Key::Divide as usize), Some(Key::Divide));
        assert_eq!(num_to_key(Key::Left as usize), Some(Key::Left));
        assert_eq!(num_to_key(Key::Right as usize), Some(Key::Right));
        assert_eq!(num_to_key(Key::Up as usize), Some(Key::Up));
        assert_eq!(num_to_key(Key::Down as usize), Some(Key::Down));
        assert_eq!(num_to_key(Key::Numpad0 as usize), Some(Key::Numpad0));
        assert_eq!(num_to_key(Key::Numpad1 as usize), Some(Key::Numpad1));
        assert_eq!(num_to_key(Key::Numpad2 as usize), Some(Key::Numpad2));
        assert_eq!(num_to_key(Key::Numpad3 as usize), Some(Key::Numpad3));
        assert_eq!(num_to_key(Key::Numpad4 as usize), Some(Key::Numpad4));
        assert_eq!(num_to_key(Key::Numpad5 as usize), Some(Key::Numpad5));
        assert_eq!(num_to_key(Key::Numpad6 as usize), Some(Key::Numpad6));
        assert_eq!(num_to_key(Key::Numpad7 as usize), Some(Key::Numpad7));
        assert_eq!(num_to_key(Key::Numpad8 as usize), Some(Key::Numpad8));
        assert_eq!(num_to_key(Key::Numpad9 as usize), Some(Key::Numpad9));
        assert_eq!(num_to_key(Key::F1 as usize), Some(Key::F1));
        assert_eq!(num_to_key(Key::F2 as usize), Some(Key::F2));
        assert_eq!(num_to_key(Key::F3 as usize), Some(Key::F3));
        assert_eq!(num_to_key(Key::F4 as usize), Some(Key::F4));
        assert_eq!(num_to_key(Key::F5 as usize), Some(Key::F5));
        assert_eq!(num_to_key(Key::F6 as usize), Some(Key::F6));
        assert_eq!(num_to_key(Key::F7 as usize), Some(Key::F7));
        assert_eq!(num_to_key(Key::F8 as usize), Some(Key::F8));
        assert_eq!(num_to_key(Key::F9 as usize), Some(Key::F9));
        assert_eq!(num_to_key(Key::F10 as usize), Some(Key::F10));
        assert_eq!(num_to_key(Key::F11 as usize), Some(Key::F11));
        assert_eq!(num_to_key(Key::F12 as usize), Some(Key::F12));
        assert_eq!(num_to_key(Key::F13 as usize), Some(Key::F13));
        assert_eq!(num_to_key(Key::F14 as usize), Some(Key::F14));
        assert_eq!(num_to_key(Key::F15 as usize), Some(Key::F15));
        assert_eq!(num_to_key(Key::Pause as usize), Some(Key::Pause));
    }
}
