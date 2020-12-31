use inle_gfx_backend::render::Font;
use std::error::Error;

pub fn load_font_from_file<'a>(fname: &str) -> Result<Font<'a>, Box<dyn Error>> {
    Font::from_file(fname).ok_or_else(|| error!())
}
