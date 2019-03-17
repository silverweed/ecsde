use super::common;
use super::common::String_Id;
use super::env::Env_Info;
use sfml::audio::{Sound, SoundBuffer};
use sfml::graphics::Texture;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub struct Resources {
    textures: HashMap<common::String_Id, Texture>,
    sounds: HashMap<common::String_Id, SoundBuffer>,
    fallback_texture: Texture,
}

impl Resources {
    pub fn new() -> Resources {
        Resources {
            textures: HashMap::new(),
            sounds: HashMap::new(),
            fallback_texture: Self::create_fallback_texture(),
        }
    }

    pub fn load_texture(&mut self, fname: &str) -> &Texture {
        let id = String_Id::from(fname);
        match self.textures.entry(id) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                if let Some(texture) = Texture::from_file(fname) {
                    v.insert(texture)
                } else {
                    eprintln!("Error loading texture {}!", fname);
                    &self.fallback_texture
                }
            }
        }
    }

    pub fn n_loaded_textures(&self) -> usize {
        self.textures.len()
    }

    fn create_fallback_texture() -> Texture {
        let pixels: [u8; 4] = [255, 10, 250, 255];
        let mut fallback_texture = Texture::new(1, 1).expect("Failed to create fallback texture!");
        fallback_texture.update_from_pixels(&pixels, 1, 1, 0, 0);
        fallback_texture.set_repeated(true);
        fallback_texture.set_smooth(false);
        fallback_texture
    }

    pub fn load_sound(&mut self, fname: &str) -> Option<&SoundBuffer> {
        let id = String_Id::from(fname);
        match self.sounds.entry(id) {
            Entry::Occupied(o) => Some(o.into_mut()),
            Entry::Vacant(v) => {
                if let Some(sound) = SoundBuffer::from_file(fname) {
                    Some(v.insert(sound))
                } else {
                    eprintln!("Error loading sound {}!", fname);
                    None // No fallback for sounds as it wouldn't make a lot of sense
                }
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use sfml::system::Vector2u;

    #[test]
    fn test_load_texture_cache() {
        // TODO come up with a better test
        let tex_name = "yv.png";
        let mut res = Resources::new();
        let env = Env_Info::gather().expect("Failed to gather env!");

        assert_eq!(res.n_loaded_textures(), 0);

        res.load_texture(&tex_path(&env, &tex_name));
        assert_eq!(res.n_loaded_textures(), 1);

        res.load_texture(&tex_path(&env, &tex_name));
        assert_eq!(res.n_loaded_textures(), 1);
    }

    #[test]
    fn test_fallback_texture() {
        let tex_name = "NOT EXISTING";
        let mut res = Resources::new();
        let env = Env_Info::gather().expect("Failed to gather env!");

        let tex = res.load_texture(tex_name);
        assert_eq!(
            tex as *const Texture,
            &res.fallback_texture as *const Texture
        );
    }
}
