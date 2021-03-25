use inle_audio_backend::sound::Sound_Buffer;
use std::error::Error;
use std::path::Path;

pub fn load_sound_buffer_from_file<'a>(fname: &Path) -> Result<Sound_Buffer<'a>, Box<dyn Error>> {
    Sound_Buffer::from_file(fname.to_str().unwrap()).ok_or_else(|| error!())
}
