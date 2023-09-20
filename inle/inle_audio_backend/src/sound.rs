#[cfg(feature = "audio-null")]
pub mod null;

#[cfg(feature = "audio-null")]
pub use self::null as backend;

pub type Sound_Buffer<'a> = backend::Sound_Buffer<'a>;
pub type Sound<'a> = backend::Sound<'a>;
