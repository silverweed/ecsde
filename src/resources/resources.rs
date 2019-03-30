mod cache;
mod sprite_storage;

use crate::core::env::Env_Info;
use sprite_storage::Sprite_Storage;

pub type Sprite_Handle = sprite_storage::Sprite_Handle;
pub type Texture_Handle = cache::Texture_Handle;
pub type Sound_Handle = cache::Sound_Handle;

pub struct Resources {
    cache: cache::Cache,
    sprite_storage: Sprite_Storage,
}

impl Resources {
    pub fn new() -> Self {
        Resources {
            cache: cache::Cache::new(),
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
        self.sprite_storage.new_sprite(handle)
    }

    pub fn destroy_sprite(&mut self, sprite: Sprite_Handle) {
        self.sprite_storage.destroy_sprite(sprite);
    }
}

// TODO when we have a frame temp allocator, this should probably allocate there.
pub fn asset_path(env: &Env_Info, dir: &str, file: &str) -> String {
    let mut s = String::from(env.get_assets_root());
    s.push('/');
    s.push_str(dir);
    s.push('/');
    s.push_str(file);
    s
}

pub fn tex_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "textures", file)
}

pub fn sound_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "sounds", file)
}
