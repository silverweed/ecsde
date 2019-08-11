use super::recording_thread;
use super::replay_data::Replay_Data_Point;
use crate::cfg;
use crate::core::common::Maybe_Error;
use crate::input::bindings::joystick::Joystick_Axis;
use crate::input::input_system::Input_Raw_Event;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

#[derive(Copy, Clone)]
pub struct Replay_Recording_System_Config {
    pub ms_per_frame: u16,
}

pub struct Replay_Recording_System {
    cur_frame: u64,
    config: Replay_Recording_System_Config,
    data_tx: Sender<Replay_Data_Point>,
    data_rx: Option<Receiver<Replay_Data_Point>>,
    recording_thread_handle: Option<JoinHandle<()>>,
}

impl Replay_Recording_System {
    pub fn new(cfg: &Replay_Recording_System_Config) -> Replay_Recording_System {
        let (data_tx, data_rx) = mpsc::channel();
        Replay_Recording_System {
            cur_frame: 0,
            config: *cfg,
            data_rx: Some(data_rx),
            data_tx,
            recording_thread_handle: None,
        }
    }

    pub fn start_recording_thread(&mut self, config: &cfg::Config) -> Maybe_Error {
        let data_rx = self.data_rx.take().unwrap();
        let file_write_interval_secs =
            *config.get_var_float_or("engine/debug/replay/file_write_interval", 1.0);
        let cfg = recording_thread::Recording_Thread_Config {
            recording_cfg: self.config,
            // @Temporary
            output_file: std::path::Path::new("replay.bin")
                .to_path_buf()
                .into_boxed_path(),
            file_write_interval: std::time::Duration::from_millis(
                (file_write_interval_secs * 1000.0) as u64,
            ),
        };
        self.recording_thread_handle =
            Some(recording_thread::start_recording_thread(data_rx, cfg)?);
        Ok(())
    }

    pub fn update(
        &mut self,
        events: &[Input_Raw_Event],
        axes: &[f32; Joystick_Axis::_Count as usize],
    ) {
        self.cur_frame += 1;
        // @Incomplete: only send data if data changed
        self.data_tx
            .send(Replay_Data_Point::new(self.cur_frame, events, axes))
            .unwrap_or_else(|err| panic!("Failed to send game actions to replay thread: {}", err));
    }
}
