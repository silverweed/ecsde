use inle_gfx_backend::render::{self, Font};
use std::error::Error;
use rusttype as rt;

pub fn load_font_from_file<'a>(fname: &str) -> Result<Font<'a>, Box<dyn Error>> {
    let bytes = std::fs::read(fname)?;
    let font = rt::Font::try_from_vec(bytes).ok_or_else(|| error!())?;

    ldebug!("Loaded font {:#?}", font);

    Ok(render::new_font(font))
}
