#[cfg(feature = "use-sfml")]
mod sfml;

#[cfg(feature = "use-sfml")]
use self::sfml as backend;

pub type Key = backend::Key;

pub(super) fn string_to_key(s: &str) -> Option<Key> {
    backend::string_to_key(s)
}
