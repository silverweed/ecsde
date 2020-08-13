#[cfg(feature = "audio-sfml")]
pub mod sfml;

#[cfg(feature = "audio-null")]
pub mod null;

#[cfg(feature = "audio-sfml")]
pub use self::sfml as backend;

#[cfg(feature = "audio-null")]
pub use self::null as backend;

pub type Sound_Buffer<'a> = backend::Sound_Buffer<'a>;
pub type Sound<'a> = backend::Sound<'a>;
