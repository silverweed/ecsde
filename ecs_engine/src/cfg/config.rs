use super::parsing::{Cfg_Entry, Raw_Config};
use super::value::Cfg_Value;
use crate::core::common::stringid::String_Id;
use std::collections::HashMap;
use std::convert::From;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::vec::Vec;

pub struct Config {
    change_rx: Receiver<Cfg_Entry>,
    change_tx: Option<Sender<Cfg_Entry>>,
    cfg_var_table: HashMap<String_Id, Cfg_Value>,
}

impl Config {
    pub fn new_from_dir(dir_path: &Path) -> Config {
        #[cfg(debug_assertions)]
        let start_t = std::time::Instant::now();
        let mut cfg_var_table = HashMap::new();
        {
            let raw = Raw_Config::new_from_dir(dir_path);

            // Flatten section/entries into string ids and convert values to cfg vars
            for section in raw.sections.into_iter() {
                for entry in section.entries.into_iter() {
                    let name = format!("{}/{}", section.header, entry.key);
                    let id = String_Id::from(name.as_str());
                    ldebug!("Loading cfg var {} = {:?}", name, entry.value);

                    cfg_var_table.insert(id, entry.value);
                }
            }
        }

        #[cfg(debug_assertions)]
        {
            let diff = start_t.elapsed();
            lok!(
                "Loaded cfg dir {:?} in {} ms.",
                dir_path,
                crate::core::time::to_secs_frac(&diff) * 1000.0,
            );
        }

        let (change_tx, change_rx) = mpsc::channel();

        Config {
            change_rx,
            change_tx: Some(change_tx),
            cfg_var_table,
        }
    }

    pub(super) fn get_change_interface(&mut self) -> Sender<Cfg_Entry> {
        self.change_tx
            .take()
            .expect("[ ERROR ] Called get_change_interface twice!")
    }

    pub fn update(&mut self) {
        let changes = self.change_rx.try_iter().collect::<Vec<Cfg_Entry>>();
        for change in changes {
            self.change_entry_value(&change.key, change.value);
        }
    }

    pub(super) fn read_cfg(&self, id: String_Id) -> Option<&Cfg_Value> {
        self.cfg_var_table.get(&id)
    }

    #[cfg(debug_assertions)]
    pub(super) fn write_cfg(&mut self, id: String_Id, val: Cfg_Value) {
        self.cfg_var_table.insert(id, val);
    }

    fn change_entry_value(&mut self, var_path: &str, value: Cfg_Value) {
        let id = String_Id::from(var_path);
        // @Incomplete: maybe give a warning if type changes?
        self.cfg_var_table.insert(id, value);
    }
}
