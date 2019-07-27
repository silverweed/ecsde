#[cfg(feature = "use-sfml")]
mod sfml;

#[cfg(feature = "use-sfml")]
pub type Key = sfml::Key;

#[cfg(feature = "use-sfml")]
pub(super) fn string_to_key(s: &str) -> Option<Key> {
    self::sfml::string_to_key(s)
}
