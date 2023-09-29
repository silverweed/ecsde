use crate::sound::Sound_Buffer;
use std::error::Error;
use std::path::Path;

pub fn load_sound_buffer_from_file(fname: &Path) -> Result<Sound_Buffer, Box<dyn Error>> {
    let sound_buf = inle_audio_backend::sound::create_sound_buffer(fname)?;
    Ok(sound_buf)
}
