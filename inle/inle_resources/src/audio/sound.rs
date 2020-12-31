use inle_audio_backend::sound::Sound_Buffer;
use std::error::Error;

pub fn load_sound_buffer_from_file<'a>(fname: &str) -> Result<Sound_Buffer<'a>, Box<dyn Error>> {
    Sound_Buffer::from_file(fname).ok_or_else(|| error!())
}
