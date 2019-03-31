extern crate stb_image;

use crate::core::common::stringid::String_Id;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use stb_image::image;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub type Texture_Handle = Option<String_Id>;
pub type Sound_Handle = Option<String_Id>;

pub type Texture_Creator = sdl2::render::TextureCreator<sdl2::video::WindowContext>;

// TODO
pub type SoundBuffer = ();

pub struct Cache {
    textures: HashMap<String_Id, Texture>,
    sounds: HashMap<String_Id, SoundBuffer>,
    fallback_texture: Texture,
    texture_creator: Texture_Creator,
}

impl Cache {
    pub fn new(texture_creator: Texture_Creator) -> Self {
        let fallback_texture = Self::create_fallback_texture(&texture_creator);
        Cache {
            texture_creator,
            textures: HashMap::new(),
            sounds: HashMap::new(),
            fallback_texture,
        }
    }

    pub fn load_texture(&mut self, fname: &str) -> Texture_Handle {
        let id = String_Id::from(fname);
        match self.textures.entry(id) {
            Entry::Occupied(_) => Some(id),
            Entry::Vacant(v) =>
            //let surface = sdl2::surface::Surface::load_bmp(fname).unwrap();
            //if let Ok(texture) = self.texture_creator.create_texture_from_surface(surface) {
            //eprintln!("Loaded texture {}", fname);
            //v.insert(texture);
            //Some(id)
            //} else {
            //eprintln!("Error loading texture {}!", fname);
            //None
            //}
            {
                match image::load(fname) {
                    image::LoadResult::Error(msg) => {
                        eprintln!("Failed to load {}: {}", fname, msg);
                        None
                    }
                    image::LoadResult::ImageU8(mut img) => {
                        let pitch = (img.width * img.depth + 3) & !3;
                        let pixel_masks = sdl2::pixels::PixelMasks {
                            bpp: 32,
                            rmask: 0x000000FF,
                            gmask: 0x0000FF00,
                            bmask: 0x00FF0000,
                            amask: if img.depth == 4 { 0xFF000000 } else { 0 },
                        };
                        if let Ok(surface) = sdl2::surface::Surface::from_data_pixelmasks(
                            img.data.as_mut_slice(),
                            img.width as u32,
                            img.height as u32,
                            pitch as u32,
                            pixel_masks,
                        ) {
                            if let Ok(texture) =
                                self.texture_creator.create_texture_from_surface(surface)
                            {
                                eprintln!("Loaded texture {}", fname);
                                v.insert(texture);
                                Some(id)
                            } else {
                                eprintln!("Error loading texture {}!", fname);
                                None
                            }
                        } else {
                            eprintln!("Failed to load surface {}!", fname);
                            None
                        }
                    }
                    image::LoadResult::ImageF32(_) => {
                        eprintln!("Unsupported format for {}", fname);
                        None
                    }
                }
            }
        }
    }

    pub fn n_loaded_textures(&self) -> usize {
        self.textures.len()
    }

    pub fn get_texture(&self, handle: Texture_Handle) -> &Texture {
        if let Some(id) = handle {
            &self.textures[&id]
        } else {
            &self.fallback_texture
        }
    }

    fn create_fallback_texture(texture_creator: &Texture_Creator) -> Texture {
        let pixels: [u8; 4] = [255, 10, 250, 255];
        let mut fallback_texture = texture_creator
            .create_texture_static(Some(sdl2::pixels::PixelFormatEnum::RGBA8888), 1, 1)
            .expect("Failed to create fallback texture!");
        if let Err(msg) = fallback_texture.update(None, &pixels, 4) {
            eprintln!("Failed to update fallback texture: {}", msg);
        }
        // TODO set this texture as repeated (probably need to use raw openGL)
        //fallback_texture.set_repeated(true);
        fallback_texture
    }

    pub fn load_sound(&mut self, fname: &str) -> Sound_Handle {
        let id = String_Id::from(fname);
        match self.sounds.entry(id) {
            Entry::Occupied(_) => Some(id),
            Entry::Vacant(v) => {
                Some(id) // TODO
                         //if let Some(sound) = SoundBuffer::from_file(fname) {
                         //Some(v.insert(sound))
                         //} else {
                         //eprintln!("Error loading sound {}!", fname);
                         //None // No fallback for sounds as it wouldn't make a lot of sense
                         //}
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
        let tex = cache.get_texture(tex);
        assert_eq!(
            tex as *const Texture,
            &cache.fallback_texture as *const Texture
        );
    }
}
