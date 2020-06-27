use super::replay_data::{Replay_Data, Replay_Data_Iter};
use crate::input::input_state::Input_Raw_State;
use std::iter::Peekable;

pub struct Replay_Input_Provider {
    replay_data_iter: Peekable<Replay_Data_Iter>,
}

impl Replay_Input_Provider {
    pub fn new(replay_data: Replay_Data) -> Self {
        Self {
            replay_data_iter: replay_data.into_iter().peekable(),
        }
    }

    pub fn has_more_input(&mut self) -> bool {
        self.replay_data_iter.peek().is_some()
    }

    pub fn get_replayed_input_for_frame(&mut self, cur_frame: u64) -> Option<Input_Raw_State> {
        if let Some(datum) = self.replay_data_iter.peek() {
            if cur_frame == datum.frame_number {
                // We have a new replay data point at this frame.
                let replay_data = self.replay_data_iter.next().unwrap();
                Some(replay_data.into())
            } else {
                assert!(cur_frame < datum.frame_number);
                None
            }
        } else {
            None
        }
    }
}
