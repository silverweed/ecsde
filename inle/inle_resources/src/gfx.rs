use std::path::Path;

pub fn load_image_from_file(fname: &Path) -> Result<Image, Box<dyn Error>> {
    // @Incomplete: we may want to choose different decoders. For now, png is the only option.
    let decoder = png::Decoder::new(File::open(fname)?);
    let (info, mut reader) = decoder.read_info()?;

    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf)?;

    lverbose!("loaded image with info {:#?}", info);

    Ok(render::new_image_with_data(
        info.width,
        info.height,
        png_to_engine_color_type(info.color_type),
        info.bit_depth as u8,
        buf,
    ))
}

pub fn load_texture_from_file(fname: &Path) -> Result<Texture, Box<dyn Error>> {
    let image = load_image_from_file(fname)?;
    Ok(render::new_texture_from_image(&image, None))
}
