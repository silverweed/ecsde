#[cfg(feature = "use-sfml")]
mod sfml;

#[cfg(feature = "use-sfml")]
use self::sfml as backend;

#[cfg(feature = "use-sfml")]
pub type Rect<T> = backend::Rect<T>;
