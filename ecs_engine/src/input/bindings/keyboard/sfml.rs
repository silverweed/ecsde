use sfml::window::Event;

pub(super) type Framework_Key = sfml::window::Key;

// @WaitForStable: make this const
pub(super) fn framework_to_engine_key(key: Framework_Key) -> Option<super::Key> {
    // Note: our Key enum is the same as SFML
    const_assert!(std::mem::size_of::<Framework_Key>() == 4);
    debug_assert!(key as u32 <= std::u8::MAX as u32); // @Robustness: should use Key_Underlying_Type
    unsafe { std::mem::transmute(key as u8) }
}

// @WaitForStable: make this const
fn engine_to_framework_key(key: super::Key) -> Framework_Key {
    // Note: our Key enum is the same as SFML
    unsafe { std::mem::transmute(key as u32) }
}

// @WaitForStable: make this const
#[inline(always)]
pub fn keypressed(code: super::Key) -> Event {
    Event::KeyPressed {
        code: engine_to_framework_key(code),
        alt: false,
        ctrl: false,
        shift: false,
        system: false,
    }
}

// @WaitForStable: make this const
#[inline(always)]
pub fn keyreleased(code: super::Key) -> Event {
    Event::KeyReleased {
        code: engine_to_framework_key(code),
        alt: false,
        ctrl: false,
        shift: false,
        system: false,
    }
}
