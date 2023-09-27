use inle_audio_backend::sound::backend;

// A Sound stands to a Sound_Buffer like a Sprite to a Texture.
pub type Sound = backend::Sound;
pub type Sound_Buffer = backend::Sound_Buffer;

pub fn play_sound(sound: &mut Sound) {
    backend::play_sound(sound);
}

pub fn sound_playing(sound: &Sound) -> bool {
    backend::sound_playing(sound)
}

pub fn create_sound_with_buffer(buf: &Sound_Buffer) -> Sound {
    backend::create_sound_with_buffer(buf)
}
