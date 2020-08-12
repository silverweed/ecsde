use super::recording_thread;
use super::replay_data::{Replay_Data_Point, Replay_Joystick_Data};
use crate::cfg::{self, Cfg_Var};
use crate::common::Maybe_Error;
use crate::core::env::Env_Info;
use crate::core::rand::Default_Rng_Seed;
use crate::input::input_state::Input_Raw_State;
use crate::input::joystick::{self, Joystick_Axis};
use crate::input::joystick_state::{self, Real_Axes_Values};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

#[derive(Copy, Clone)]
pub struct Replay_Recording_System_Config {
    pub ms_per_frame: f32,
    pub rng_seed: Default_Rng_Seed,
}

pub struct Replay_Recording_System {
    config: Replay_Recording_System_Config,
    data_tx: Sender<Replay_Data_Point>,
    data_rx: Option<Receiver<Replay_Data_Point>>,
    recording_thread_handle: Option<JoinHandle<()>>,
    prev_axes_values: [Real_Axes_Values; joystick::JOY_COUNT as usize],
}

impl Replay_Recording_System {
    pub fn new(cfg: Replay_Recording_System_Config) -> Replay_Recording_System {
        let (data_tx, data_rx) = mpsc::channel();
        Replay_Recording_System {
            config: cfg,
            data_rx: Some(data_rx),
            data_tx,
            recording_thread_handle: None,
            prev_axes_values: std::default::Default::default(),
        }
    }

    pub fn start_recording_thread(&mut self, env: &Env_Info, cfg: &cfg::Config) -> Maybe_Error {
        let data_rx = self
            .data_rx
            .take()
            .unwrap_or_else(|| panic!("start_recording_thread called twice!"));

        let file_write_interval_secs =
            Cfg_Var::<f32>::new("engine/debug/replay/file_write_interval", cfg);
        let output_file = Cfg_Var::<String>::new("engine/debug/replay/out_file", cfg);

        let mut output_file_path = PathBuf::from(env.working_dir.clone());
        output_file_path.push(output_file.read(cfg));

        linfo!("Recording input to {:?}", output_file_path);

        let rec_cfg = recording_thread::Recording_Thread_Config {
            recording_cfg: self.config,
            output_file: output_file_path.into_boxed_path(),
            file_write_interval: std::time::Duration::from_millis(
                (file_write_interval_secs.read(cfg) * 1000.0) as u64,
            ),
        };

        match recording_thread::start_recording_thread(data_rx, rec_cfg) {
            Ok(handle) => self.recording_thread_handle = Some(handle),
            Err(err) => lerr!("start_recording_thread failed with err {}", err),
        }

        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        self.data_rx.is_none()
    }

    /// Note: joy_mask tells which values of `axes` must be considered.
    pub fn update(&mut self, input_raw_state: &Input_Raw_State, cur_frame: u64) {
        let mut should_send = !input_raw_state.events.is_empty();
        let mut joy_data: [Replay_Joystick_Data; joystick::JOY_COUNT as usize] =
            std::default::Default::default();

        let (axes, joy_mask) = joystick_state::all_joysticks_values(&input_raw_state.joy_state);

        for (i, axes) in axes.iter().enumerate() {
            if (joy_mask & (1 << i)) == 0 {
                continue;
            }

            joy_data[i].axes = *axes;
            joy_data[i].axes_mask = !0;
            should_send = true;

            self.prev_axes_values[i] = *axes;
        }

        if should_send {
            self.data_tx
                .send(Replay_Data_Point::new(
                    cur_frame,
                    &input_raw_state.events,
                    &joy_data,
                    joy_mask,
                ))
                .unwrap_or_else(|err| {
                    panic!("Failed to send game actions to replay thread: {}", err)
                });
        }
    }
}

/// Compares `old` with `new` and returns a bitmask with '1' for each different axis.
fn calc_axes_diff_mask(old: &Real_Axes_Values, new: &Real_Axes_Values) -> u8 {
    let mut mask = 0u8;
    for i in 0..Joystick_Axis::_Count as usize {
        mask |= u8::from((old[i] - new[i]).abs() > std::f32::EPSILON) << i;
    }
    mask
}
