use crate::core::common::stringid::String_Id;
use sdl2::image::LoadTexture;
use sdl2::render::{Texture, TextureCreator};
use sdl2::ttf::{Font, Sdl2TtfContext};
use std::convert::From;

pub trait Resource_Loader<'l, R> {
    type Args: ?Sized;

    fn load(&'l self, data: &Self::Args) -> Result<R, String>;
}

impl<'l, T> Resource_Loader<'l, Texture<'l>> for TextureCreator<T> {
    type Args = str;

    fn load(&'l self, path: &str) -> Result<Texture, String> {
        self.load_texture(path)
    }
}

impl<'l> Resource_Loader<'l, Font<'l, 'static>> for Sdl2TtfContext {
    type Args = Font_Load_Info;

    fn load(&'l self, info: &Font_Load_Info) -> Result<Font<'l, 'static>, String> {
        self.load_font(&info.path, info.size)
    }
}

#[derive(Debug)]
pub struct Font_Load_Info {
    pub path: String,
    pub size: u16,
}

impl<'a> From<&'a Font_Load_Info> for String_Id {
    fn from(fli: &Font_Load_Info) -> Self {
        String_Id::from(format!("{}_{}", fli.path, fli.size).as_str())
    }
}
