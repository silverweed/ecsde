mod cache;
mod sprite_storage;

use crate::core::env::Env_Info;
use sdl2::render::{Texture, TextureQuery};
use sprite_storage::Sprite_Storage;
use std::path::{Path, PathBuf};

pub type Sprite_Handle = sprite_storage::Sprite_Handle;
pub type Sprite = sprite_storage::Sprite;
pub type Texture_Handle = cache::Texture_Handle;
pub type Sound_Handle = cache::Sound_Handle;
type Texture_Creator = cache::Texture_Creator;

pub struct Resources {
    cache: cache::Cache,
    sprite_storage: Sprite_Storage,
}

impl Resources {
    pub fn new(texture_creator: Texture_Creator) -> Self {
        Resources {
            cache: cache::Cache::new(texture_creator),
            sprite_storage: Sprite_Storage::new(),
        }
    }

    pub fn load_texture(&mut self, fname: &str) -> Texture_Handle {
        self.cache.load_texture(fname)
    }

    pub fn n_loaded_textures(&self) -> usize {
        self.cache.n_loaded_textures()
    }

    pub fn load_sound(&mut self, fname: &str) -> Sound_Handle {
        self.cache.load_sound(fname)
    }

    pub fn n_loaded_sounds(&self) -> usize {
        self.cache.n_loaded_sounds()
    }

    pub fn new_sprite(&mut self, tex_fname: &str) -> Sprite_Handle {
        let handle = self.load_texture(tex_fname);
        let TextureQuery { width, height, .. } = self.cache.get_texture(handle).query();
        self.sprite_storage.new_sprite(handle, width, height)
    }

    pub fn destroy_sprite(&mut self, sprite: Sprite_Handle) {
        self.sprite_storage.destroy_sprite(sprite);
    }

    pub fn get_sprite(&self, handle: Sprite_Handle) -> &Sprite {
        self.sprite_storage.get_sprite(handle)
    }

    pub fn get_texture(&self, handle: Texture_Handle) -> &Texture {
        self.cache.get_texture(handle)
    }
}

// TODO when we have a frame temp allocator, this should probably allocate there.
pub fn asset_path(env: &Env_Info, dir: &str, file: &str) -> String {
    let mut s = PathBuf::from(env.get_assets_root());
    s.push(dir);
    s.push(file);
    s.into_os_string().into_string().unwrap()
}

pub fn tex_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "textures", file)
}

pub fn sound_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "sounds", file)
}
