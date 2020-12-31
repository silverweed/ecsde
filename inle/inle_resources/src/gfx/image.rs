use inle_gfx_backend::render::{self, Color_Type, Image, Texture};
use std::error::Error;
use std::fs::File;

const fn png_to_engine_color_type(c: png::ColorType) -> Color_Type {
    match c {
        png::ColorType::Grayscale => Color_Type::Grayscale,
        png::ColorType::RGB => Color_Type::RGB,
        png::ColorType::Indexed => Color_Type::Indexed,
        png::ColorType::GrayscaleAlpha => Color_Type::Grayscale_Alpha,
        png::ColorType::RGBA => Color_Type::RGBA,
    }
}

pub fn load_image_from_file(fname: &str) -> Result<Image, Box<dyn Error>> {
    // @Incomplete: we may want to choose different decoders. For now, png is the only option.
    let decoder = png::Decoder::new(File::open(fname)?);
    let (info, mut reader) = decoder.read_info()?;

    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf)?;

    ldebug!("loaded image with info {:#?}", info);

    Ok(render::new_image_with_data(
        info.width,
        info.height,
        png_to_engine_color_type(info.color_type),
        info.bit_depth as u8,
        buf,
    ))
}

pub fn load_texture_from_file<'a>(fname: &str) -> Result<Texture<'a>, Box<dyn Error>> {
    let image = load_image_from_file(fname)?;
    Ok(render::new_texture_from_image(&image, None).ok_or_else(|| error!())?)
}
