// Engine config (mapped from the cfg files)
mod parsing;

use crate::core::env::Env_Info;
use crate::resources;
use std::collections::HashMap;
use std::path::PathBuf;
use std::vec::Vec;

#[derive(Debug, PartialEq)]
enum Cfg_Value {
    Nil,
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
}

/// Contains all configurations from all cfg files.
/// Conceptually, it's as all cfg sections were in the same file: they're just split
/// into multiple files for convenience.
// @Convenience: this means all headers must be unique across files; maybe splitting
// files logically may become convenient in the long run...we'll see.
pub struct Config {
    sections: HashMap<String, Cfg_Section>,
}

/// A Cfg_Section is a section in a cfg file delimited by /header and
/// consisting of multiple lines of the format:
/// [#] key [value] [# ...]
#[derive(Debug)]
struct Cfg_Section {
    pub header: String,
    pub entries: Vec<Cfg_Entry>,
}

#[derive(Debug)]
struct Cfg_Entry {
    pub key: String,
    pub value: Cfg_Value,
}

impl Config {
    pub fn new(env: &Env_Info) -> Config {
        let sections = parsing::parse_config_dir(env.get_cfg_root()).unwrap();
        Config {
            sections: HashMap::new(),
        }
    }
}

pub fn cfg_path(env: &Env_Info, dir: &str, file: &str) -> PathBuf {
    let mut s = PathBuf::from(env.get_cfg_root());
    s.push(dir);
    s.push(file);
    s.set_extension("cfg");
    s
}
