#[macro_use]
pub mod loaders;

pub mod audio;
pub mod gfx;

use crate::core::env::Env_Info;
use std::path::PathBuf;

// @Speed: when we have a frame temp allocator, this should probably allocate there.
pub fn asset_path(env: &Env_Info, dir: &str, file: &str) -> String {
    let mut s = PathBuf::from(env.get_assets_root());
    s.push(dir);
    s.push(file);
    s.into_os_string().into_string().unwrap()
}
