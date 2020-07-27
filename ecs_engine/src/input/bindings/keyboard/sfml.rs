use super::Key;

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
