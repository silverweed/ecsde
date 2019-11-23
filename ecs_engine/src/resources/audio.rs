mod cache;

use super::asset_path;
use super::loaders;
use crate::core::env::Env_Info;

pub type Sound_Buffer = cache::Sound_Buffer;
pub type Sound<'a> = cache::Sound<'a>;
pub type Sound_Handle = loaders::Res_Handle;

pub struct Audio_Resources<'l> {
    sounds: cache::Sound_Cache<'l>,
}

impl<'l> Audio_Resources<'l> {
    pub fn new() -> Self {
        Audio_Resources {
            sounds: cache::Sound_Cache::new(),
        }
    }

    pub fn load_sound(&mut self, fname: &str) -> Sound_Handle {
        self.sounds.load(fname)
    }

    pub fn n_loaded_sounds(&self) -> usize {
        self.sounds.n_loaded()
    }

    pub fn get_sound(&self, handle: Sound_Handle) -> &Sound_Buffer {
        self.sounds.must_get(handle)
    }
}

pub fn sound_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "sounds", file)
}
