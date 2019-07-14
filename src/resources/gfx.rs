mod cache;

use super::asset_path;
use crate::core::common::stringid::String_Id;
use crate::core::env::Env_Info;
use crate::gfx::render::{Font, Texture};

pub type Texture_Handle = cache::Texture_Handle;
pub type Font_Handle = Option<String_Id>;

pub struct Gfx_Resources<'l> {
    textures: cache::Texture_Cache<'l>,
    fonts: cache::Font_Cache<'l>,
}

impl<'l> Gfx_Resources<'l> {
    pub fn new() -> Self {
        Gfx_Resources {
            textures: cache::Texture_Cache::new(),
            fonts: cache::Font_Cache::new(),
        }
    }

    pub fn load_texture(&mut self, fname: &str) -> Texture_Handle {
        self.textures.load(fname)
    }

    pub fn get_texture(&self, handle: Texture_Handle) -> &Texture<'_> {
        self.textures.must_get(handle)
    }

    pub fn get_texture_mut<'a>(&'a mut self, handle: Texture_Handle) -> &'a mut Texture<'l>
    where
        'l: 'a,
    {
        self.textures.must_get_mut(handle)
    }

    pub fn load_font(&mut self, fname: &str) -> Font_Handle {
        self.fonts.load(fname)
    }

    pub fn get_font(&self, handle: Font_Handle) -> &Font<'_> {
        assert!(handle != None, "Invalid Font_Handle in get_font!");
        self.fonts.must_get(handle)
    }
}

pub fn tex_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "textures", file)
}

pub fn font_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "fonts", file)
}
