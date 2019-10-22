// Engine config (mapped from the cfg files)
pub mod sync;

mod config;
mod parsing;
mod value;
mod var;

use crate::core::env::Env_Info;
use std::convert::{Into, TryFrom};
use std::path::PathBuf;
use typename::TypeName;
use value::Cfg_Value;

pub type Cfg_Var<T> = var::Cfg_Var<T>;
pub type Config = config::Config;

pub fn cfg_path(env: &Env_Info, dir: &str, file: &str) -> PathBuf {
    let mut s = PathBuf::from(env.get_cfg_root());
    s.push(dir);
    s.push(file);
    s.set_extension("cfg");
    s
}

#[inline(always)]
pub fn from_cfg<T>(var: Cfg_Var<T>) -> T
where
    T: Default + TypeName + Into<Cfg_Value> + TryFrom<Cfg_Value>,
{
    var::from_cfg(var)
}
