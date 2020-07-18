#[cfg(feature = "win-sfml")]
pub mod sfml;

#[cfg(feature = "win-glfw")]
pub mod glfw;

#[cfg(feature = "win-sfml")]
use self::sfml as backend;

#[cfg(feature = "win-glfw")]
use self::glfw as backend;

pub use backend::should_event_be_serialized;
