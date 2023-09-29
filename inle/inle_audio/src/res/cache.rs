use crate::sound::Sound_Buffer;
use inle_resources::loaders;

define_file_loader!(
    Sound_Buffer,
    Sound_Loader,
    Sound_Cache,
    super::sound::load_sound_buffer_from_file
);
