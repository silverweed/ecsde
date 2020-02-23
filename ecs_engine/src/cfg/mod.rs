// Engine config (mapped from the cfg files)
pub mod sync;

mod config;
mod parsing;
mod value;
mod var;

use crate::core::env::Env_Info;
use std::path::PathBuf;

pub type Cfg_Var<T> = var::Cfg_Var<T>;
pub type Config = config::Config;
#[cfg(debug_assertions)]
pub type Cfg_Value = value::Cfg_Value;

pub fn cfg_path(env: &Env_Info, dir: &str, file: &str) -> PathBuf {
    let mut s = PathBuf::from(env.get_cfg_root());
    s.push(dir);
    s.push(file);
    s.set_extension("cfg");
    s
}
