#[cfg(feature = "use-sfml")]
pub mod sfml;

#[cfg(feature = "use-sfml")]
use self::sfml as backend;

pub type Key = backend::Key;

pub fn string_to_key(s: &str) -> Option<Key> {
    backend::string_to_key(s)
}

pub fn num_to_key(num: usize) -> Option<Key> {
    backend::num_to_key(num)
}
