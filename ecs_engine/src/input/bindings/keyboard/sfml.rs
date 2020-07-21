use super::{Input_Raw_Event, Key};
use sfml::window::Event;

pub(super) type Framework_Key = sfml::window::Key;

// @WaitForStable: make this const
pub(super) fn framework_to_engine_key(key: Framework_Key) -> Option<Key> {
    // Note: our Key enum is the same as SFML
    const_assert!(std::mem::size_of::<Framework_Key>() == 4);
    debug_assert!(key as u32 <= std::u8::MAX as u32); // @Robustness: should use Key_Underlying_Type
    unsafe { std::mem::transmute(key as u8) }
}

// @WaitForStable: make this const
fn engine_to_framework_key(key: Key) -> Framework_Key {
    // Note: our Key enum is the same as SFML
    unsafe { std::mem::transmute(key as u32) }
}

// @WaitForStable: make this const
#[inline(always)]
pub fn keypressed(code: Key) -> Event {
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
pub fn keyreleased(code: Key) -> Event {
    Event::KeyReleased {
        code: engine_to_framework_key(code),
        alt: false,
        ctrl: false,
        shift: false,
        system: false,
    }
}

pub(super) fn update_kb_state(kb_state: &mut super::Keyboard_State, events: &[Input_Raw_Event]) {
    use crate::input::bindings::modifiers::*;
    for evt in events {
        match evt {
            Input_Raw_Event::KeyPressed { code, .. } => {
                if let Some(code) = framework_to_engine_key(*code) {
                    match code {
                        Key::LControl => kb_state.modifiers_pressed |= MOD_LCTRL,
                        Key::RControl => kb_state.modifiers_pressed |= MOD_RCTRL,
                        Key::LShift => kb_state.modifiers_pressed |= MOD_LSHIFT,
                        Key::RShift => kb_state.modifiers_pressed |= MOD_RSHIFT,
                        Key::LAlt => kb_state.modifiers_pressed |= MOD_LALT,
                        Key::RAlt => kb_state.modifiers_pressed |= MOD_RALT,
                        Key::LSystem => kb_state.modifiers_pressed |= MOD_LSUPER,
                        Key::RSystem => kb_state.modifiers_pressed |= MOD_RSUPER,
                        _ => {}
                    }
                    kb_state.keys_pressed.insert(code);
                }
            }
            Input_Raw_Event::KeyReleased { code, .. } => {
                if let Some(code) = framework_to_engine_key(*code) {
                    match code {
                        Key::LControl => kb_state.modifiers_pressed &= !MOD_LCTRL,
                        Key::RControl => kb_state.modifiers_pressed &= !MOD_RCTRL,
                        Key::LShift => kb_state.modifiers_pressed &= !MOD_LSHIFT,
                        Key::RShift => kb_state.modifiers_pressed &= !MOD_RSHIFT,
                        Key::LAlt => kb_state.modifiers_pressed &= !MOD_LALT,
                        Key::RAlt => kb_state.modifiers_pressed &= !MOD_RALT,
                        Key::LSystem => kb_state.modifiers_pressed &= !MOD_LSUPER,
                        Key::RSystem => kb_state.modifiers_pressed &= !MOD_RSUPER,
                        _ => {}
                    }
                    kb_state.keys_pressed.remove(&code);
                }
            }
            _ => (),
        }
    }
}
