use crate::core::common;
use crate::core::common::String_Id;
use crate::core::env::Env_Info;
use sfml::audio::{Sound, SoundBuffer};
use sfml::graphics::{Texture, TextureRef};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub type Texture_Handle = Option<String_Id>;

pub struct Cache {
    textures: HashMap<String_Id, Texture>,
    sounds: HashMap<String_Id, SoundBuffer>,
    fallback_texture: Texture,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            textures: HashMap::new(),
            sounds: HashMap::new(),
            fallback_texture: Self::create_fallback_texture(),
        }
    }

    pub fn load_texture(&mut self, fname: &str) -> Texture_Handle {
        let id = String_Id::from(fname);
        match self.textures.entry(id) {
            //Entry::Occupied(o) => o.into_mut(),
            //Entry::Vacant(v) => {
            //if let Some(texture) = Texture::from_file(fname) {
            //v.insert(texture)
            //} else {
            //eprintln!("Error loading texture {}!", fname);
            //&self.fallback_texture
            //}
            //}
            Entry::Occupied(o) => Some(id),
            Entry::Vacant(v) => {
                if let Some(texture) = Texture::from_file(fname) {
                    v.insert(texture);
                    Some(id)
                } else {
                    eprintln!("Error loading texture {}!", fname);
                    //&self.fallback_texture
                    None
                }
            }
        }
    }

    pub fn n_loaded_textures(&self) -> usize {
        self.textures.len()
    }

    pub fn get_texture(&self, handle: &Texture_Handle) -> &Texture {
        if let Some(id) = handle {
            &self.textures[id]
        } else {
            &self.fallback_texture
        }
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

    pub fn n_loaded_sounds(&self) -> usize {
        self.sounds.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_texture() {
        let tex_name = "NOT EXISTING";
        let mut cache = Cache::new();
        let env = Env_Info::gather().expect("Failed to gather env!");

        let tex = cache.load_texture(tex_name);
        assert_eq!(
            tex as *const Texture,
            &cache.fallback_texture as *const Texture
        );
    }
}
