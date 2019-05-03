mod cache;
pub mod loaders;

use self::cache::{Font_Manager, Sound_Manager, Texture_Manager};
use self::loaders::Font_Load_Info;
use crate::audio::sound_loader::Sound_Loader;
use crate::core::common::stringid::String_Id;
use crate::core::env::Env_Info;
use sdl2::pixels::Color;
use sdl2::render::Texture;
use sdl2::ttf::{Font, Sdl2TtfContext};
use std::path::PathBuf;

pub type Texture_Handle = Option<String_Id>;
pub type Sound_Handle = Option<String_Id>;
pub type Font_Handle = Option<String_Id>;
pub type Sound_Buffer = cache::Sound_Buffer;
type Texture_Creator = cache::Texture_Creator;

pub struct Resources<'l> {
    textures: Texture_Manager<'l>,
    fonts: Font_Manager<'l>,
    sounds: Sound_Manager<'l>,
}

impl<'l> Resources<'l> {
    pub fn new(
        texture_creator: &'l Texture_Creator,
        ttf: &'l Sdl2TtfContext,
        sound_loader: &'l Sound_Loader,
    ) -> Self {
        Resources {
            textures: Texture_Manager::new(&texture_creator),
            fonts: Font_Manager::new(&ttf),
            sounds: Sound_Manager::new(&sound_loader),
        }
    }

    pub fn load_texture(&mut self, fname: &str) -> Texture_Handle {
        self.textures.load(fname).unwrap_or_else(|msg| {
            eprintln!("{}", msg);
            None
        })
    }

    pub fn get_texture<'a>(&self, handle: Texture_Handle) -> &Texture<'a>
    where
        'l: 'a,
    {
        self.textures.must_get(handle)
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

    pub fn get_font<'a>(&self, handle: Font_Handle) -> &Font<'a, 'static>
    where
        'l: 'a,
    {
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

    pub fn load_sound(&mut self, fname: &str) -> Sound_Handle {
        self.sounds.load(fname).unwrap_or_else(|msg| {
            eprintln!("{}", msg);
            None
        })
    }

    pub fn n_loaded_sounds(&self) -> usize {
        self.sounds.n_loaded()
    }

    pub fn get_sound(&self, handle: Sound_Handle) -> Sound_Buffer {
        self.sounds.must_get(handle).clone()
    }
}

// @Speed: when we have a frame temp allocator, this should probably allocate there.
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

pub fn font_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "fonts", file)
}
