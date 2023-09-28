use super::loaders;
use inle_audio_backend::sound::{Audio_Context, Sound_Buffer};
use inle_core::env::{asset_path, Env_Info};
use std::path::Path;

mod cache;
mod sound;

pub type Sound_Handle = loaders::Res_Handle;

pub struct Audio_Resources {
    audio_ctx: Audio_Context,
    sounds: cache::Sound_Cache,
}

impl Audio_Resources {
    pub fn new() -> Self {
        let audio_ctx = inle_audio_backend::sound::init_audio();
        Audio_Resources {
            audio_ctx,
            sounds: cache::Sound_Cache::new(),
        }
    }

    pub fn load_sound(&mut self, fname: &Path) -> Sound_Handle {
        self.sounds.load(fname)
    }

    pub fn n_loaded_sounds(&self) -> usize {
        self.sounds.n_loaded()
    }

    pub fn get_sound_buffer(&self, handle: Sound_Handle) -> &Sound_Buffer {
        self.sounds.must_get(handle)
    }

    pub fn get_sound_buffer_mut(&mut self, handle: Sound_Handle) -> &mut Sound_Buffer {
        self.sounds.must_get_mut(handle)
    }
}

pub fn sound_path(env: &Env_Info, file: &str) -> Box<Path> {
    asset_path(env, "sounds", file)
}
