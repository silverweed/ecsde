use crate::sound::{Audio_Context, Sound_Buffer};
use inle_core::env::{asset_path, Env_Info};
use inle_resources::loaders;
use std::path::Path;

mod cache;
mod sound;

pub type Sound_Buffer_Handle = loaders::Res_Handle;

pub struct Audio_Resources {
    audio_ctx: Audio_Context,
    cache: cache::Sound_Cache,
}

impl Audio_Resources {
    pub fn new() -> Self {
        let audio_ctx = inle_audio_backend::sound::init_audio();
        Audio_Resources {
            audio_ctx,
            cache: cache::Sound_Cache::new(),
        }
    }

    pub fn load_sound(&mut self, fname: &Path) -> Sound_Buffer_Handle {
        self.cache.load(fname)
    }

    pub fn n_loaded_cache(&self) -> usize {
        self.cache.n_loaded()
    }

    pub fn get_sound_buffer(&self, handle: Sound_Buffer_Handle) -> &Sound_Buffer {
        self.cache.must_get(handle)
    }

    pub fn get_sound_buffer_mut(&mut self, handle: Sound_Buffer_Handle) -> &mut Sound_Buffer {
        self.cache.must_get_mut(handle)
    }
}

pub fn sound_path(env: &Env_Info, file: &str) -> Box<Path> {
    asset_path(env, "sounds", file)
}
