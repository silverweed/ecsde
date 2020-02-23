#[cfg(feature = "use-sfml")]
pub mod sfml;

#[cfg(feature = "use-sfml")]
use self::sfml as backend;

pub type Key = backend::Key;

pub fn num_to_key(num: usize) -> Option<Key> {
    backend::num_to_key(num)
}

pub fn is_key_pressed(key: Key) -> bool {
    backend::is_key_pressed(key)
}

#[cfg(debug_assertions)]
// Note: this assumes an english layout
pub fn key_to_char(key: Key, maiusc: bool) -> Option<char> {
    match key {
        Key::A => Some(if maiusc { 'A' } else { 'a' }),
        Key::B => Some(if maiusc { 'B' } else { 'b' }),
        Key::C => Some(if maiusc { 'C' } else { 'c' }),
        Key::D => Some(if maiusc { 'D' } else { 'd' }),
        Key::E => Some(if maiusc { 'E' } else { 'e' }),
        Key::F => Some(if maiusc { 'F' } else { 'f' }),
        Key::G => Some(if maiusc { 'G' } else { 'g' }),
        Key::H => Some(if maiusc { 'H' } else { 'h' }),
        Key::I => Some(if maiusc { 'I' } else { 'i' }),
        Key::J => Some(if maiusc { 'J' } else { 'j' }),
        Key::K => Some(if maiusc { 'K' } else { 'k' }),
        Key::L => Some(if maiusc { 'L' } else { 'l' }),
        Key::M => Some(if maiusc { 'M' } else { 'm' }),
        Key::N => Some(if maiusc { 'N' } else { 'n' }),
        Key::O => Some(if maiusc { 'O' } else { 'o' }),
        Key::P => Some(if maiusc { 'P' } else { 'p' }),
        Key::Q => Some(if maiusc { 'Q' } else { 'q' }),
        Key::R => Some(if maiusc { 'R' } else { 'r' }),
        Key::S => Some(if maiusc { 'S' } else { 's' }),
        Key::T => Some(if maiusc { 'T' } else { 't' }),
        Key::U => Some(if maiusc { 'U' } else { 'u' }),
        Key::V => Some(if maiusc { 'V' } else { 'v' }),
        Key::W => Some(if maiusc { 'W' } else { 'w' }),
        Key::X => Some(if maiusc { 'X' } else { 'x' }),
        Key::Y => Some(if maiusc { 'Y' } else { 'y' }),
        Key::Z => Some(if maiusc { 'Z' } else { 'z' }),
        Key::Num0 => Some(if maiusc { ')' } else { '0' }),
        Key::Num1 => Some(if maiusc { '!' } else { '1' }),
        Key::Num2 => Some(if maiusc { '@' } else { '2' }),
        Key::Num3 => Some(if maiusc { '#' } else { '3' }),
        Key::Num4 => Some(if maiusc { '$' } else { '4' }),
        Key::Num5 => Some(if maiusc { '%' } else { '5' }),
        Key::Num6 => Some(if maiusc { '^' } else { '6' }),
        Key::Num7 => Some(if maiusc { '&' } else { '7' }),
        Key::Num8 => Some(if maiusc { '*' } else { '8' }),
        Key::Num9 => Some(if maiusc { '(' } else { '9' }),
        Key::LBracket => Some(if maiusc { '{' } else { '[' }),
        Key::RBracket => Some(if maiusc { '}' } else { ']' }),
        Key::SemiColon => Some(if maiusc { ':' } else { ';' }),
        Key::Comma => Some(if maiusc { '<' } else { ',' }),
        Key::Period => Some(if maiusc { '>' } else { '.' }),
        Key::Quote => Some(if maiusc { '\'' } else { '"' }),
        Key::Slash => Some(if maiusc { '?' } else { '/' }),
        Key::BackSlash => Some(if maiusc { '|' } else { '\\' }),
        Key::Tilde => Some(if maiusc { '~' } else { '`' }),
        Key::Equal => Some(if maiusc { '+' } else { '=' }),
        Key::Dash => Some(if maiusc { '_' } else { '-' }),
        Key::Space => Some(' '),
        Key::Tab => Some('\t'),
        Key::Add => Some('+'),
        Key::Subtract => Some('-'),
        Key::Multiply => Some('*'),
        Key::Divide => Some('/'),
        Key::Numpad0 => Some('0'),
        Key::Numpad1 => Some('1'),
        Key::Numpad2 => Some('2'),
        Key::Numpad3 => Some('3'),
        Key::Numpad4 => Some('4'),
        Key::Numpad5 => Some('5'),
        Key::Numpad6 => Some('6'),
        Key::Numpad7 => Some('7'),
        Key::Numpad8 => Some('8'),
        Key::Numpad9 => Some('9'),
        _ => None,
    }
}

pub fn string_to_key(s: &str) -> Option<Key> {
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
