use super::parsing::{Cfg_Entry, Raw_Config};
use super::value::Cfg_Value;
use crate::core::common::stringid::String_Id;
use std::collections::HashMap;
use std::convert::From;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::RwLock;
use std::vec::Vec;

lazy_static! {
    pub static ref CFG_VAR_TABLE: RwLock<HashMap<String_Id, Cfg_Value>> =
        RwLock::new(HashMap::new());
}

pub struct Config {
    change_rx: Receiver<Cfg_Entry>,
    change_tx: Option<Sender<Cfg_Entry>>,
}

impl Config {
    pub fn new_from_dir(dir_path: &Path) -> Config {
        #[cfg(debug_assertions)]
        let start_t = std::time::Instant::now();

        {
            let raw = Raw_Config::new_from_dir(dir_path);
            let mut table = CFG_VAR_TABLE.write().unwrap();

            // Flatten section/entries into string ids and convert values to cfg vars
            for section in raw.sections.into_iter() {
                for entry in section.entries.into_iter() {
                    let name = format!("{}/{}", section.header, entry.key);
                    let id = String_Id::from(name.as_str());
                    eprintln!("Loading cfg var {} = {:?}", name, entry.value);

                    table.insert(id, entry.value);
                }
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
