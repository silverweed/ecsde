#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate inle_common;

#[macro_use]
extern crate inle_diagnostics;

#[macro_use]
pub mod loaders;

pub mod audio;
pub mod gfx;

use inle_core::env::Env_Info;
use std::path::PathBuf;

// @Speed: when we have a frame temp allocator, this should probably allocate there.
pub fn asset_path(env: &Env_Info, dir: &str, file: &str) -> String {
    let mut s = PathBuf::from(env.assets_root.as_ref());
    s.push(dir);
    s.push(file);
    s.into_os_string().into_string().unwrap()
}
