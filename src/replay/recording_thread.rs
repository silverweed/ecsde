use super::recording_system::Replay_Recording_System_Config;
use super::replay_data::Replay_Data_Point;
use crate::core::common::serialize::{Binary_Serializable, Byte_Stream};
use crate::core::common::Maybe_Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, SystemTime};

pub struct Recording_Thread_Config {
    pub output_file: Box<Path>,
    pub file_write_interval: Duration,
    pub recording_cfg: Replay_Recording_System_Config,
}

pub fn start_recording_thread(
    recv: Receiver<Replay_Data_Point>,
    cfg: Recording_Thread_Config,
) -> std::io::Result<thread::JoinHandle<()>> {
    thread::Builder::new()
        .name(String::from("recording_thread"))
        .spawn(move || recording_loop(recv, &cfg).unwrap())
}

fn recording_loop(recv: Receiver<Replay_Data_Point>, cfg: &Recording_Thread_Config) -> Maybe_Error {
    let mut file = File::create(&cfg.output_file)?;
    write_prelude(&mut file, &cfg.recording_cfg)?;

    let mut replay_data_buffer = vec![];
    let mut time_elapsed = Duration::new(0, 0);
    loop {
        let start = SystemTime::now();

        for point in recv.try_iter() {
            replay_data_buffer.push(point);
        }

        time_elapsed += SystemTime::now()
            .duration_since(start)
            .unwrap_or_else(|err| panic!("Failed to get duration_since: {}", err));

        if time_elapsed > cfg.file_write_interval {
            write_record_data(&mut file, &replay_data_buffer)?;
            replay_data_buffer.clear();
            time_elapsed = Duration::new(0, 0);
        }
    }
}

fn write_prelude(
    file: &mut File,
    recording_cfg: &Replay_Recording_System_Config,
) -> std::io::Result<()> {
    let mut byte_stream = Byte_Stream::new();
    byte_stream.write_u16(recording_cfg.ms_per_frame);
    file.write_all(byte_stream.as_ref())
}

fn write_record_data(file: &mut File, data: &[Replay_Data_Point]) -> std::io::Result<()> {
    let mut byte_stream = Byte_Stream::new();
    for point in data.iter() {
        point.serialize(&mut byte_stream)?;
    }
    file.write_all(byte_stream.as_ref())
}
