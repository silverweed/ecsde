use super::parsing::{Cfg_Entry, Cfg_Value, Raw_Config};
use super::Cfg_Var;
use crate::core::common::stringid::String_Id;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::From;
use std::path::Path;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::vec::Vec;

pub struct Config {
    bool_vars: HashMap<String_Id, Rc<RefCell<bool>>>,
    int_vars: HashMap<String_Id, Rc<RefCell<i32>>>,
    float_vars: HashMap<String_Id, Rc<RefCell<f32>>>,
    string_vars: HashMap<String_Id, Rc<RefCell<String>>>,
    change_rx: Receiver<Cfg_Entry>,
    change_tx: Option<Sender<Cfg_Entry>>,
}

impl Config {
    #[cfg(test)]
    pub fn new_empty() -> Config {
        let (change_tx, change_rx) = mpsc::channel();
        Config {
            bool_vars: HashMap::new(),
            int_vars: HashMap::new(),
            float_vars: HashMap::new(),
            string_vars: HashMap::new(),
            change_rx,
            change_tx: Some(change_tx),
        }
    }

    pub fn new_from_dir(dir_path: &Path) -> Config {
        let raw = Raw_Config::new_from_dir(dir_path);

        // Flatten section/entries into string ids and convert values to cfg vars
        let mut bool_vars = HashMap::new();
        let mut int_vars = HashMap::new();
        let mut float_vars = HashMap::new();
        let mut string_vars = HashMap::new();
        for section in raw.sections.into_iter() {
            for entry in section.entries.into_iter() {
                let name = format!("{}/{}", section.header, entry.key);
                let id = String_Id::from(name.as_str());
                eprintln!("Loading cfg var {} = {:?}", name, entry.value);
                match entry.value {
                    Cfg_Value::Bool(v) => {
                        bool_vars.insert(id, Rc::new(RefCell::new(v)));
                    }
                    Cfg_Value::Int(v) => {
                        int_vars.insert(id, Rc::new(RefCell::new(v)));
                    }
                    Cfg_Value::Float(v) => {
                        float_vars.insert(id, Rc::new(RefCell::new(v)));
                    }
                    Cfg_Value::String(v) => {
                        string_vars.insert(id, Rc::new(RefCell::new(v)));
                    }
                    _ => (),
                }
            }
        }

        let (change_tx, change_rx) = mpsc::channel();

        Config {
            bool_vars,
            int_vars,
            float_vars,
            string_vars,
            change_rx,
            change_tx: Some(change_tx),
        }
    }

    pub(super) fn get_change_interface(&mut self) -> Sender<Cfg_Entry> {
        self.change_tx
            .take()
            .expect("[ ERROR ] Called get_change_interface twice!")
    }

    pub fn get_var_bool(&self, path: &str) -> Option<Cfg_Var<bool>> {
        let id = String_Id::from(path);
        self.bool_vars.get(&id).map(|v| Cfg_Var::new(&v))
    }

    pub fn get_var_int(&self, path: &str) -> Option<Cfg_Var<i32>> {
        let id = String_Id::from(path);
        self.int_vars.get(&id).map(|v| Cfg_Var::new(&v))
    }

    pub fn get_var_float(&self, path: &str) -> Option<Cfg_Var<f32>> {
        let id = String_Id::from(path);
        self.float_vars.get(&id).map(|v| Cfg_Var::new(&v))
    }

    pub fn get_var_string(&self, path: &str) -> Option<Cfg_Var<String>> {
        let id = String_Id::from(path);
        self.string_vars.get(&id).map(|v| Cfg_Var::new(&v))
    }

    pub fn get_var_bool_or(&self, path: &str, default: bool) -> Cfg_Var<bool> {
        let var = self.get_var_bool(path);
        if let Some(var) = var {
            var
        } else {
            eprintln!(
                "Notice: could not find bool cfg_var {}: using default {}",
                path, default
            );
            Cfg_Var::new_from_val(default)
        }
    }

    pub fn get_var_int_or(&self, path: &str, default: i32) -> Cfg_Var<i32> {
        let var = self.get_var_int(path);
        if let Some(var) = var {
            var
        } else {
            eprintln!(
                "Notice: could not find int cfg_var {}: using default {}",
                path, default
            );
            Cfg_Var::new_from_val(default)
        }
    }

    pub fn get_var_float_or(&self, path: &str, default: f32) -> Cfg_Var<f32> {
        let var = self.get_var_float(path);
        if let Some(var) = var {
            var
        } else {
            eprintln!(
                "Notice: could not find float cfg_var {}: using default {}",
                path, default
            );
            Cfg_Var::new_from_val(default)
        }
    }

    pub fn get_var_string_or(&self, path: &str, default: &str) -> Cfg_Var<String> {
        let var = self.get_var_string(path);
        if let Some(var) = var {
            var
        } else {
            eprintln!(
                "Notice: could not find string cfg_var {}: using default {}",
                path, default
            );
            Cfg_Var::new_from_val(String::from(default))
        }
    }

    pub fn update(&mut self) {
        let changes = self.change_rx.try_iter().collect::<Vec<Cfg_Entry>>();
        for change in changes {
            self.change_entry_value(&change.key, change.value);
        }
    }

    fn change_entry_value(&mut self, var_path: &str, value: Cfg_Value) {
        let id = String_Id::from(var_path);
        match value {
            Cfg_Value::Bool(v) => {
                if let Some(var) = self.bool_vars.get_mut(&id) {
                    var.replace(v);
                } else {
                    eprintln!(
                        "Notice: tried to update value for inexisting cfg var {}",
                        var_path
                    );
                }
            }
            Cfg_Value::Int(v) => {
                if let Some(var) = self.int_vars.get_mut(&id) {
                    var.replace(v);
                } else {
                    eprintln!(
                        "Notice: tried to update value for inexisting cfg var {}",
                        var_path
                    );
                }
            }
            Cfg_Value::Float(v) => {
                if let Some(var) = self.float_vars.get_mut(&id) {
                    var.replace(v);
                } else {
                    eprintln!(
                        "Notice: tried to update value for inexisting cfg var {}",
                        var_path
                    );
                }
            }
            Cfg_Value::String(v) => {
                if let Some(var) = self.string_vars.get_mut(&id) {
                    var.replace(v);
                } else {
                    eprintln!(
                        "Notice: tried to update value for inexisting cfg var {}",
                        var_path
                    );
                }
            }
            _ => (),
        }
    }
}
