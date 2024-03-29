use crate::loaders;
use inle_audio_backend::sound::Sound_Buffer;

define_file_loader!(
    Sound_Buffer,
    Sound_Loader,
    Sound_Cache,
    super::sound::load_sound_buffer_from_file
);
