use super::parsing::Raw_Config;
use super::value::Cfg_Value;
use crate::common::stringid::String_Id;
use std::collections::HashMap;
use std::convert::From;
use std::path::Path;

#[cfg(debug_assertions)]
use {
    super::parsing::Cfg_Entry,
    std::sync::mpsc::{self, Receiver, Sender},
};

pub struct Config {
    cfg_var_table: HashMap<String_Id, Cfg_Value>,

    #[cfg(debug_assertions)]
    change_rx: Receiver<Cfg_Entry>,
    #[cfg(debug_assertions)]
    change_tx: Option<Sender<Cfg_Entry>>,
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
                diff.as_secs_f32() * 1000.0,
            );
        }

        #[cfg(debug_assertions)]
        let (change_tx, change_rx) = mpsc::channel();

        Config {
            cfg_var_table,
            #[cfg(debug_assertions)]
            change_rx,
            #[cfg(debug_assertions)]
            change_tx: Some(change_tx),
        }
    }
}

#[cfg(debug_assertions)]
impl Config {
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

    pub fn read_cfg(&self, id: String_Id) -> Option<&Cfg_Value> {
        self.cfg_var_table.get(&id)
    }

    pub fn write_cfg(&mut self, id: String_Id, val: Cfg_Value) {
        self.cfg_var_table.insert(id, val);
    }

    fn change_entry_value(&mut self, var_path: &str, value: Cfg_Value) {
        let id = String_Id::from(var_path);
        // @Incomplete: maybe give a warning if type changes?
        self.cfg_var_table.insert(id, value);
    }

    pub fn get_all_pairs(&self) -> impl Iterator<Item = (String, Cfg_Value)> + '_ {
        self.cfg_var_table
            .iter()
            .map(|(key, val)| (key.to_string(), val.clone()))
    }
}

#[cfg(not(debug_assertions))]
impl Config {
    pub(super) fn read_cfg(&self, id: String_Id) -> Option<&Cfg_Value> {
        self.cfg_var_table.get(&id)
    }
}

