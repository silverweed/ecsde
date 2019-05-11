// Engine config (mapped from the cfg files)
mod parsing;
mod var;

use var::{Cfg_Var, Cfg_Var_Type};

use crate::core::env::Env_Info;
use crate::resources;
use std::collections::HashMap;
use std::convert::From;
use std::path::PathBuf;
use std::vec::Vec;

#[derive(Debug, PartialEq)]
// @Cleanup: this type is pub because we need it to expose Cfg_Var_Type. Maybe find a way to expose less.
pub enum Cfg_Value {
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
    pub entries: HashMap<String, Cfg_Entry>,
}

#[derive(Debug)]
struct Cfg_Entry {
    pub key: String,
    pub value: Cfg_Value,
}

impl Config {
    pub fn new(path: &std::path::Path) -> Config {
        let sections_list = parsing::parse_config_dir(path).unwrap();
        let mut sections = HashMap::new();
        for section in sections_list.into_iter() {
            sections.insert(String::from(section.header.as_str()), section);
        }
        Config { sections }
    }

    /// Gets a config variable via a path of the form: section/entry.
    pub fn get_var<T: Cfg_Var_Type>(&self, path: &str) -> Option<Cfg_Var<T::Type>> {
        let tokens: Vec<&str> = path.split('/').collect();
        if tokens.len() == 2 {
            let (section_name, entry_name) = (tokens[0], tokens[1]);
            let section = self.sections.get(section_name)?;
            let entry = section.entries.get(entry_name)?;
            if T::is_type(&entry.value) {
                Some(Cfg_Var::new(T::value(&entry.value)))
            } else {
                eprintln!("Cfg var {} found, but its type is not the right one!", path);
                None
            }
        } else {
            eprintln!("Cfg var not found: {}", path);
            None
        }
    }

    pub fn get_var_or<T: Cfg_Var_Type, D>(&self, path: &str, default: D) -> Cfg_Var<T::Type>
    where
        T::Type: From<D>,
    {
        if let Some(var) = self.get_var::<T>(path) {
            var
        } else {
            Cfg_Var::new(default.into())
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
