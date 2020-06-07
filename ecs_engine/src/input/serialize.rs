#[cfg(feature = "use-sfml")]
pub mod sfml;

#[cfg(feature = "use-sfml")]
use self::sfml as backend;

pub use backend::should_event_be_serialized;
