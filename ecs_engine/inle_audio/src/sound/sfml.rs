use sfml::audio as sfaud;

pub type Sound<'a> = sfaud::Sound<'a>;

sf_wrap!(Sound_Buffer, sfml::audio::SoundBuffer);

pub fn play_sound(sound: &mut Sound) {
    sound.play();
}

pub fn sound_playing(sound: &Sound) -> bool {
    sound.status() == sfaud::SoundStatus::Playing
}

pub fn create_sound_with_buffer<'a>(buf: &'a Sound_Buffer) -> Sound<'a> {
    Sound::with_buffer(buf)
}
