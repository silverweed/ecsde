use super::parsing::Raw_Config;
use super::value::Cfg_Value;
use inle_common::stringid::String_Id;
use std::collections::{hash_map::Entry, HashMap, HashSet};
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
    just_changed: HashSet<String_Id>,

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
                    lverbose!("Loading cfg var {} = {:?}", name, entry.value);

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
            just_changed: HashSet::default(),
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
        self.just_changed.clear();

        let changes = self.change_rx.try_iter().collect::<Vec<Cfg_Entry>>();
        for change in changes {
            self.change_entry_value(&change.key, change.value);
        }
    }

    pub fn read_cfg(&self, id: String_Id) -> Option<&Cfg_Value> {
        self.cfg_var_table.get(&id)
    }

    pub fn write_cfg(&mut self, id: String_Id, val: Cfg_Value) -> Result<(), String> {
        match self.cfg_var_table.entry(id) {
            Entry::Vacant(v) => {
                v.insert(val);
            }
            Entry::Occupied(mut o) => {
                if std::mem::discriminant(&val) == std::mem::discriminant(o.get()) {
                    o.insert(val);
                } else {
                    return Err(format!("Cfg_Var {:?} was not updated because its current value ({:?}) has a type different from the new one ({:?}).", id, o.get(), val));
                }
            }
        }
        self.just_changed.insert(id);
        Ok(())
    }

    // Only works if the cfg var is a bool
    pub fn toggle_cfg(&mut self, id: String_Id) -> Result<(), String> {
        match self.cfg_var_table.entry(id) {
            Entry::Vacant(_v) => {
                return Err(format!("Cfg_Var {:?} does not exist.", id));
            }
            Entry::Occupied(mut o) => {
                if let Cfg_Value::Bool(val) = o.get() {
                    o.insert(Cfg_Value::Bool(!val));
                } else {
                    return Err(format!("Cfg_Var {:?} was not updated because its current value ({:?}) has a type different from Bool.", id, o.get()));
                }
            }
        }
        self.just_changed.insert(id);
        Ok(())
    }

    fn change_entry_value(&mut self, var_path: &str, value: Cfg_Value) {
        let id = String_Id::from(var_path);
        if let Err(msg) = self.write_cfg(id, value) {
            lwarn!("{}", msg);
        }
    }

    pub fn get_all_pairs(&self) -> impl Iterator<Item = (String, Cfg_Value)> + '_ {
        self.cfg_var_table
            .iter()
            .map(|(key, val)| (key.to_string(), val.clone()))
    }

    pub fn has_changed(&self, id: String_Id) -> bool {
        self.just_changed.contains(&id)
    }
}

#[cfg(not(debug_assertions))]
impl Config {
    pub(super) fn read_cfg(&self, id: String_Id) -> Option<&Cfg_Value> {
        self.cfg_var_table.get(&id)
    }
}
