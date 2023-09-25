#[cfg(feature = "audio-null")]
pub mod null;

#[cfg(feature = "audio-null")]
pub use self::null as backend;

pub type Sound_Buffer = backend::Sound_Buffer;
pub type Sound = backend::Sound;
