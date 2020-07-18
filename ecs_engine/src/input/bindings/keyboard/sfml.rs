use sfml::window::Event;

pub(super) type Framework_Key = sfml::window::Key;

pub(super) fn framework_to_engine_key(key: Framework_Key) -> Option<super::Key> {
    // Note: our Key enum is the same as SFML
    unsafe { std::mem::transmute(key) }
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
