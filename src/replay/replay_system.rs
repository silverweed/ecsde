use super::replay_data::Replay_Data;
use crate::core::common::Maybe_Error;
use crate::core::input::Action_List;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub struct Replay_System {
    cur_frame: u64,
    prev_action_list: Action_List,
    data: Replay_Data,
}

impl Replay_System {
    pub fn new() -> Replay_System {
        Replay_System {
            cur_frame: 0,
            prev_action_list: Action_List::default(),
            data: Replay_Data::new(),
        }
    }

    pub fn update(&mut self, list: &Action_List) {
        self.cur_frame += 1;

        let new_directions = list.get_directions();
        if new_directions != self.prev_action_list.get_directions() {
            self.data.add_point(self.cur_frame, new_directions);
            self.prev_action_list = list.clone();
        }
    }

    pub fn serialize(&self, file_path: &Path) -> Maybe_Error {
        let mut file = File::create(file_path)?;
        file.write_all(self.data.serialize().as_bytes())?;
        Ok(())
    }
}
