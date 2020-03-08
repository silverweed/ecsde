use super::recording_thread;
use super::replay_data::{Replay_Data_Point, Replay_Joystick_Data};
use crate::cfg::{self, Cfg_Var};
use crate::common::Maybe_Error;
use crate::input::bindings::joystick::{self, Joystick_Axis};
use crate::input::input_system::Input_Raw_Event;
use crate::input::joystick_state::Real_Axes_Values;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

#[derive(Copy, Clone)]
pub struct Replay_Recording_System_Config {
    pub ms_per_frame: f32,
}

pub struct Replay_Recording_System {
    cur_frame: u64,
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
            cur_frame: 0,
            config: cfg,
            data_rx: Some(data_rx),
            data_tx,
            recording_thread_handle: None,
            prev_axes_values: std::default::Default::default(),
        }
    }

    pub fn start_recording_thread(&mut self, cfg: &cfg::Config) -> Maybe_Error {
        let data_rx = self
            .data_rx
            .take()
            .unwrap_or_else(|| panic!("start_recording_thread called twice!"));

        let file_write_interval_secs =
            Cfg_Var::<f32>::new("engine/debug/replay/file_write_interval", cfg);
        let output_file = Cfg_Var::<String>::new("engine/debug/replay/out_file", cfg);

        let cfg = recording_thread::Recording_Thread_Config {
            recording_cfg: self.config,
            output_file: std::path::Path::new(&output_file.read(cfg))
                .to_path_buf()
                .into_boxed_path(),
            file_write_interval: std::time::Duration::from_millis(
                (file_write_interval_secs.read(cfg) * 1000.0) as u64,
            ),
        };

        self.recording_thread_handle =
            Some(recording_thread::start_recording_thread(data_rx, cfg)?);

        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        self.data_rx.is_none()
    }

    /// Note: joy_mask tells which values of `axes` must be considered.
    pub fn update(
        &mut self,
        events: &[Input_Raw_Event],
        axes: &[Real_Axes_Values; joystick::JOY_COUNT as usize],
        joy_mask: u8,
    ) {
        self.cur_frame += 1;

        let mut should_send = !events.is_empty();
        let mut joy_data: [Replay_Joystick_Data; joystick::JOY_COUNT as usize] =
            std::default::Default::default();

        for (i, axes) in axes.iter().enumerate() {
            if (joy_mask & (1 << i)) == 0 {
                continue;
            }

            let axes_mask = calc_axes_diff_mask(&self.prev_axes_values[i], axes);
            if axes_mask != 0 {
                joy_data[i].axes = *axes;
                joy_data[i].axes_mask = axes_mask;
                should_send = true;
            }

            self.prev_axes_values[i] = *axes;
        }

        if should_send {
            self.data_tx
                .send(Replay_Data_Point::new(
                    self.cur_frame,
                    events,
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
