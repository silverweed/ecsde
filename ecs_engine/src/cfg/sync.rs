use super::config::Config;
use super::parsing::{self, Cfg_Entry, Cfg_Section};
use crate::fs::{file_watcher, utils};
use notify::DebouncedEvent;
use std::sync::mpsc::Sender;

pub struct Config_Watch_Handler {
    config_change: Sender<Cfg_Entry>,
}

impl Config_Watch_Handler {
    pub fn new(config: &mut Config) -> Self {
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
                    for Cfg_Section { header, entries } in sections.into_iter() {
                        for Cfg_Entry { key, value } in entries.into_iter() {
                            let name = format!("{}/{}", header, key);
                            self.config_change
                                .send(Cfg_Entry { key: name, value })
                                .unwrap();
                        }
                    }
                }
            }
            _ => (),
        }
    }
}
