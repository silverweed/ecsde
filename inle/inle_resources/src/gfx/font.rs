use super::image;
use inle_gfx_backend::render::{self, Font, Font_Metadata, Glyph_Bounds, Glyph_Data};
use std::error::Error;
use std::path::{Path, PathBuf};

pub fn load_font_from_file(fname: &Path) -> Result<Font, Box<dyn Error>> {
    let fname = fname
        .with_extension("")
        .into_os_string()
        .into_string()
        .unwrap();
    let atlas_fname = PathBuf::from(format!("{}_msdf.png", fname));
    let metadata_fname = PathBuf::from(format!("{}_meta.csv", fname));

    let atlas_img = image::load_image_from_file(&atlas_fname)?;
    let atlas = render::new_texture_from_image(&atlas_img, None);

    let metadata_csv = std::fs::read_to_string(&metadata_fname)?;
    let metadata = parse_font_metadata_from_csv(&metadata_csv, render::get_texture_size(&atlas));

    Ok(Font { atlas, metadata })
}

fn parse_font_metadata_from_csv(csv: &str, atlas_size: (u32, u32)) -> Font_Metadata {
    let mut metadata = Font_Metadata::with_atlas_size(atlas_size.0, atlas_size.1);

    for line in csv.lines() {
        let toks: Vec<_> = line.split(',').collect();
        // expected line:
        // glyph_id, advance, plane_bounds_l, b, r, t, atlas_bounds_l, b, r, t
        if toks.len() != 10 {
            lwarn!("Weird line in font csv metadata: {}", line);
            continue;
        }

        if let Some((glyph_id, data)) = parse_font_metadata_csv_line(&toks, atlas_size) {
            metadata.add_glyph_data(glyph_id, data);
        }
    }

    metadata
}

fn parse_font_metadata_csv_line(
    toks: &[&str],
    (atlas_w, atlas_h): (u32, u32),
) -> Option<(char, Glyph_Data)> {
    let glyph = toks[0].parse::<u8>().ok()?;
    let advance = toks[1].parse::<f32>().ok()?;
    let plane_bounds_l = toks[2].parse::<f32>().ok()?;
    let plane_bounds_b = toks[3].parse::<f32>().ok()?;
    let plane_bounds_r = toks[4].parse::<f32>().ok()?;
    let plane_bounds_t = toks[5].parse::<f32>().ok()?;
    let atlas_bounds_l = toks[6].parse::<f32>().ok()?;
    let atlas_bounds_b = toks[7].parse::<f32>().ok()?;
    let atlas_bounds_r = toks[8].parse::<f32>().ok()?;
    let atlas_bounds_t = toks[9].parse::<f32>().ok()?;

    Some((
        glyph as char,
        Glyph_Data {
            advance,
            plane_bounds: Glyph_Bounds {
                left: plane_bounds_l,
                bot: plane_bounds_b,
                right: plane_bounds_r,
                top: plane_bounds_t,
            },
            normalized_atlas_bounds: Glyph_Bounds {
                left: atlas_bounds_l / atlas_w as f32,
                bot: (atlas_h as f32 - atlas_bounds_b) / atlas_h as f32,
                right: atlas_bounds_r / atlas_w as f32,
                top: (atlas_h as f32 - atlas_bounds_t) / atlas_h as f32,
            },
        },
    ))
}
