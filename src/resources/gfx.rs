use super::asset_path;
use super::cache;
use super::loaders::Font_Load_Info;
use crate::core::common::colors::Color;
use crate::core::common::stringid::String_Id;
use crate::core::env::Env_Info;
use crate::gfx::render::Texture;
use sdl2::ttf::{Font, Sdl2TtfContext};

pub type Texture_Handle = Option<String_Id>;
pub type Texture_Creator = cache::Texture_Creator;
pub type Font_Handle = Option<String_Id>;

pub struct Gfx_Resources<'l> {
    textures: cache::Texture_Manager<'l>,
    fonts: cache::Font_Manager<'l>,
}

impl<'l> Gfx_Resources<'l> {
    pub fn new(texture_creator: &'l Texture_Creator, ttf: &'l Sdl2TtfContext) -> Self {
        Gfx_Resources {
            textures: cache::Texture_Manager::new(&texture_creator),
            fonts: cache::Font_Manager::new(&ttf),
        }
    }

    pub fn load_texture(&mut self, fname: &str) -> Texture_Handle {
        self.textures.load(fname).unwrap_or_else(|msg| {
            eprintln!("{}", msg);
            None
        })
    }

    pub fn get_texture(&self, handle: Texture_Handle) -> &Texture<'l> {
        self.textures.must_get(handle)
    }

    pub fn get_texture_mut(&mut self, handle: Texture_Handle) -> &mut Texture<'l> {
        self.textures.must_get_mut(handle)
    }

    pub fn n_loaded_textures(&self) -> usize {
        self.textures.n_loaded()
    }

    pub fn load_font(&mut self, fname: &str, size: u16) -> Font_Handle {
        self.fonts
            .load(&Font_Load_Info {
                path: String::from(fname),
                size,
            })
            .unwrap_or_else(|msg| {
                eprintln!("{}", msg);
                None
            })
    }

    pub fn get_font(&self, handle: Font_Handle) -> &Font<'_, 'static> {
        self.fonts.must_get(handle)
    }

    pub fn n_loaded_fonts(&self) -> usize {
        self.fonts.n_loaded()
    }

    pub fn create_font_texture(
        &mut self,
        txt: &str,
        font_handle: Font_Handle,
        color: Color,
    ) -> Texture_Handle {
        self.textures
            .create_font_texture(&self.fonts, txt, font_handle, color)
    }
}

pub fn tex_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "textures", file)
}

pub fn font_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "fonts", file)
}
