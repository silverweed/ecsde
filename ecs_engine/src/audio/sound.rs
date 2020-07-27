#[cfg(feature = "audio-sfml")]
mod sfml;

#[cfg(feature = "audio-null")]
mod null;

#[cfg(feature = "audio-sfml")]
use self::sfml as backend;

#[cfg(feature = "audio-null")]
use self::null as backend;

pub type Sound_Buffer<'a> = backend::Sound_Buffer<'a>;
pub type Sound<'a> = backend::Sound<'a>;

pub fn play_sound(sound: &mut Sound) {
    backend::play_sound(sound);
}

pub fn sound_playing(sound: &Sound) -> bool {
    backend::sound_playing(sound)
}

pub fn create_sound_with_buffer<'a>(buf: &'a Sound_Buffer) -> Sound<'a> {
    backend::create_sound_with_buffer(buf)
}
