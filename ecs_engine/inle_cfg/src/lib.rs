#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate inle_diagnostics;

mod sync;
mod config;
mod parsing;
mod value;
mod var;

use inle_core::env::Env_Info;
use std::path::PathBuf;

pub type Cfg_Var<T> = var::Cfg_Var<T>;
pub type Config = config::Config;
#[cfg(debug_assertions)]
pub type Cfg_Value = value::Cfg_Value;

pub fn cfg_path(env: &Env_Info, dir: &str, file: &str) -> PathBuf {
    let mut s = PathBuf::from(env.cfg_root.as_ref());
    s.push(dir);
    s.push(file);
    s.set_extension("cfg");
    s
}
