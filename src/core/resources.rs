use super::common;
use super::common::String_Id;
use super::env::Env_Info;
use sfml::audio;
use sfml::graphics::Texture;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub struct Resources {
    textures: HashMap<common::String_Id, Texture>,
    fallback_texture: Texture,
}

impl Resources {
    pub fn new() -> Resources {
        Resources {
            textures: HashMap::new(),
            fallback_texture: Texture::new(10, 10).expect("Failed to create fallback texture!"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_texture_cache() {
        let tex_name = "yv.png";
        let mut res = Resources::new();
        let env = Env_Info::gather().expect("Failed to gather env!");

        assert_eq!(res.n_loaded_textures(), 0);

        res.load_texture(&tex_path(&env, &tex_name));
        assert_eq!(res.n_loaded_textures(), 1);

        res.load_texture(&tex_path(&env, &tex_name));
        assert_eq!(res.n_loaded_textures(), 1);
    }
}
