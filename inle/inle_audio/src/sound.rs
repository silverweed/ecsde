use inle_audio_backend::sound::backend;

pub type Sound<'a> = backend::Sound<'a>;
pub type Sound_Buffer<'a> = backend::Sound_Buffer<'a>;

pub fn play_sound(sound: &mut Sound) {
    backend::play_sound(sound);
}

pub fn sound_playing(sound: &Sound) -> bool {
    backend::sound_playing(sound)
}

pub fn create_sound_with_buffer<'a>(buf: &'a Sound_Buffer) -> Sound<'a> {
    backend::create_sound_with_buffer(buf)
}
