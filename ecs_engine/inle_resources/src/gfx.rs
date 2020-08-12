mod cache;

use super::asset_path;
use super::loaders;
use crate::core::env::Env_Info;
use crate::gfx::render::{self, Font, Shader, Texture};

pub type Texture_Handle = loaders::Res_Handle;
pub type Font_Handle = loaders::Res_Handle;
pub type Shader_Handle = loaders::Res_Handle;

pub struct Gfx_Resources<'l> {
    textures: cache::Texture_Cache<'l>,
    fonts: cache::Font_Cache<'l>,
}

impl<'l> Gfx_Resources<'l> {
    pub fn new() -> Self {
        if !render::shaders_are_available() {
            lwarn!("This platform does not support shaders.");
        }

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

pub struct Shader_Cache<'l>(cache::Shader_Cache<'l>);

impl<'l> Shader_Cache<'l> {
    pub fn new() -> Self {
        Self(cache::Shader_Cache::new())
    }

    pub fn load_shader(&mut self, fname: &str) -> Shader_Handle {
        if render::shaders_are_available() {
            self.0.load(fname)
        } else {
            None
        }
    }

    pub fn get_shader(&self, handle: Shader_Handle) -> &Shader<'_> {
        debug_assert!(render::shaders_are_available());
        self.0.must_get(handle)
    }

    pub fn get_shader_mut<'a>(&'a mut self, handle: Shader_Handle) -> &'a mut Shader<'l>
    where
        'l: 'a,
    {
        debug_assert!(render::shaders_are_available());
        self.0.must_get_mut(handle)
    }
}

pub fn tex_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "textures", file)
}

pub fn font_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "fonts", file)
}

pub fn shader_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "shaders", file)
}
