use super::parsing::{Cfg_Entry, Raw_Config};
use super::value::Cfg_Value;
use super::Cfg_Var;
use crate::core::common::stringid::String_Id;
use std::collections::HashMap;
use std::convert::From;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::RwLock;
use std::vec::Vec;
use typename::TypeName;

lazy_static! {
    pub static ref CFG_VAR_TABLE: RwLock<HashMap<String_Id, Cfg_Value>> =
        RwLock::new(HashMap::new());
}

pub struct Config {
    change_rx: Receiver<Cfg_Entry>,
    change_tx: Option<Sender<Cfg_Entry>>,
}

impl Config {
    #[cfg(test)]
    pub fn new_empty() -> Config {
        let (change_tx, change_rx) = mpsc::channel();
        Config {
            change_rx,
            change_tx: Some(change_tx),
        }
    }

    pub fn new_from_dir(dir_path: &Path) -> Config {
        #[cfg(debug_assertions)]
        let start_t = std::time::Instant::now();

        let raw = Raw_Config::new_from_dir(dir_path);

        // Flatten section/entries into string ids and convert values to cfg vars
        for section in raw.sections.into_iter() {
            for entry in section.entries.into_iter() {
                let name = format!("{}/{}", section.header, entry.key);
                let id = String_Id::from(name.as_str());
                eprintln!("Loading cfg var {} = {:?}", name, entry.value);

                let mut table = CFG_VAR_TABLE.write().unwrap();
                table.insert(id, entry.value);
            }
        }

        #[cfg(debug_assertions)]
        {
            let diff = start_t.elapsed();
            println!(
                "[ OK ] Loaded cfg dir {:?} in {} ms.",
                dir_path,
                crate::core::time::to_secs_frac(&diff) * 1000.0
            );
        }

        let (change_tx, change_rx) = mpsc::channel();

        Config {
            change_rx,
            change_tx: Some(change_tx),
        }
    }

    pub(super) fn get_change_interface(&mut self) -> Sender<Cfg_Entry> {
        self.change_tx
            .take()
            .expect("[ ERROR ] Called get_change_interface twice!")
    }

    pub fn get_var<T>(&self, path: &str) -> Option<Cfg_Var<T>>
    where
        T: Default + std::convert::Into<Cfg_Value> + TypeName,
    {
        let id = String_Id::from(path);
        let table = CFG_VAR_TABLE.read().unwrap();
        if table.get(&id).is_some() {
            Some(Cfg_Var::new(id))
        } else {
            None
        }
    }

    pub fn get_var_or<T>(&self, path: &str, default: T) -> Cfg_Var<T>
    where
        T: Default + std::fmt::Debug + std::convert::Into<Cfg_Value> + TypeName,
    {
        let var = self.get_var(path);
        if let Some(var) = var {
            var
        } else {
            eprintln!(
                "Notice: could not find cfg_var {}: using default {:?}",
                path, default
            );
            Cfg_Var::new_from_val(default)
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
        let mut table = CFG_VAR_TABLE.write().unwrap();
        // @Incomplete: maybe give a warning if type changes?
        table.insert(id, value);
    }
}
