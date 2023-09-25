use inle_gfx_backend::render::{self, Shader};
use std::error::Error;
use std::fs;

pub fn load_shader_from_file(vert_fname: &str, frag_fname: &str) -> Result<Shader, Box<dyn Error>> {
    let vert_src = fs::read(vert_fname)?;
    let frag_src = fs::read(frag_fname)?;

    Ok(render::new_shader(
        &vert_src,
        &frag_src,
        Some(&format!("{}+{}", vert_fname, frag_fname)),
    ))
}
