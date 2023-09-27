#[cfg(feature = "audio-null")]
pub mod null;

#[cfg(feature = "audio-openal")]
pub mod openal;

#[cfg(feature = "audio-null")]
pub use self::null as backend;

#[cfg(feature = "audio-openal")]
pub use self::openal as backend;

pub type Sound_Buffer = backend::Sound_Buffer;
pub type Sound = backend::Sound;
pub type Audio_Context = backend::Audio_Context;

pub use backend::init_audio;
