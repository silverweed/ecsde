use super::parsing::{self, Cfg_Entry, Cfg_Section, Cfg_Value};
use super::{Config, Config_Change_Interface};
use crate::fs::{file_watcher, utils};
use notify::DebouncedEvent;
use std::cell::RefCell;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

pub struct Config_Watch_Handler {
    config_change: Arc<Mutex<Config_Change_Interface>>,
}

impl Config_Watch_Handler {
    pub fn new(config: &Config) -> Self {
        Config_Watch_Handler {
            config_change: config.get_change_interface(),
        }
    }
}

impl file_watcher::File_Watcher_Event_Handler for Config_Watch_Handler {
    fn handle(&mut self, event: &DebouncedEvent) {
        match event {
            DebouncedEvent::Write(ref pathbuf) | DebouncedEvent::Create(ref pathbuf)
                if !utils::is_hidden(pathbuf) =>
            {
                if let Ok(sections) = parsing::parse_config_file(pathbuf) {
                    let mut config_change = self.config_change.lock().unwrap();

                    for Cfg_Section { header, entries } in sections.into_iter() {
                        for Cfg_Entry { key, value } in entries.into_iter() {
                            let name = format!("{}/{}", header, key);
                            config_change.request_entry_change(Cfg_Entry { key: name, value });
                        }
                    }
                }
            }
            _ => (),
        }
    }
}
