use crate::core::common::stringid::String_Id;
use crate::gfx::render::{Font, Texture};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub type Texture_Handle = Option<String_Id>;
pub type Font_Handle = Option<String_Id>;

pub struct Texture_Cache<'l> {
    textures: HashMap<String_Id, Texture<'l>>,
}

impl<'l> Texture_Cache<'l> {
    pub fn new() -> Self {
        Texture_Cache {
            textures: HashMap::new(),
        }
    }

    pub fn load(&mut self, fname: &str) -> Texture_Handle {
        let id = String_Id::from(fname);
        match self.textures.entry(id) {
            Entry::Occupied(_) => Some(id),
            Entry::Vacant(v) => {
                let tex = Texture::from_file(fname).unwrap();
                v.insert(tex);
                eprintln!("Loaded texture {}", fname);
                Some(id)
            }
        }
    }

    pub fn must_get(&self, handle: Texture_Handle) -> &Texture<'_> {
        &self.textures[&handle.unwrap()]
    }

    pub fn must_get_mut<'a>(&'a mut self, handle: Texture_Handle) -> &'a mut Texture<'l>
    where
        'l: 'a,
    {
        self.textures.get_mut(&handle.unwrap()).unwrap()
    }
}

pub struct Font_Cache<'l> {
    fonts: HashMap<String_Id, Font<'l>>,
}

impl Font_Cache<'_> {
    pub fn new() -> Self {
        Font_Cache {
            fonts: HashMap::new(),
        }
    }

    pub fn load(&mut self, fname: &str) -> Font_Handle {
        let id = String_Id::from(fname);
        match self.fonts.entry(id) {
            Entry::Occupied(_) => Some(id),
            Entry::Vacant(v) => {
                let font = Font::from_file(fname).unwrap();
                v.insert(font);
                eprintln!("Loaded font {}", fname);
                Some(id)
            }
        }
    }

    pub fn must_get(&self, handle: Font_Handle) -> &Font<'_> {
        &self.fonts[&handle.unwrap()]
    }
}
