use sfml::window::Event;
pub(super) use sfml::window::Key;

// @Cleanup: does this belong here or in the frontend?
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

#[inline(always)]
pub const fn keypressed(code: Key) -> Event {
    Event::KeyPressed {
        code,
        alt: false,
        ctrl: false,
        shift: false,
        system: false,
    }
}

#[inline(always)]
pub const fn keyreleased(code: Key) -> Event {
    Event::KeyReleased {
        code,
        alt: false,
        ctrl: false,
        shift: false,
        system: false,
    }
}
