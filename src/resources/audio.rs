use super::asset_path;
use super::cache;
use crate::audio::sound_loader::Sound_Loader;
use crate::core::common::stringid::String_Id;
use crate::core::env::Env_Info;

pub type Sound_Buffer = cache::Sound_Buffer;
pub type Sound_Handle = Option<String_Id>;

pub struct Audio_Resources<'l> {
    sounds: cache::Sound_Manager<'l>,
}

impl<'l> Audio_Resources<'l> {
    pub fn new(sound_loader: &'l Sound_Loader) -> Self {
        Audio_Resources {
            sounds: cache::Sound_Manager::new(&sound_loader),
        }
    }

    pub fn load_sound(&mut self, fname: &str) -> Sound_Handle {
        self.sounds.load(fname).unwrap_or_else(|msg| {
            eprintln!("{}", msg);
            None
        })
    }

    pub fn n_loaded_sounds(&self) -> usize {
        self.sounds.n_loaded()
    }

    pub fn get_sound(&self, handle: Sound_Handle) -> Sound_Buffer {
        self.sounds.must_get(handle).clone()
    }
}

pub fn sound_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "sounds", file)
}
