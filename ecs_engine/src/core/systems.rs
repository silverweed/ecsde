use super::env::Env_Info;
use crate::audio::audio_system;
use crate::cfg;
use crate::debug::tracer;
use crate::input::input_system;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(debug_assertions)]
use crate::debug::debug_ui_system;
#[cfg(debug_assertions)]
use crate::replay::recording_system;

pub struct Core_Systems<'r> {
    pub input_system: input_system::Input_System,
    pub audio_system: audio_system::Audio_System<'r>,
}

#[cfg(debug_assertions)]
pub struct Debug_Systems {
    pub debug_ui_system: debug_ui_system::Debug_Ui_System,
    pub replay_recording_system: recording_system::Replay_Recording_System,
    pub tracer: Rc<RefCell<tracer::Debug_Tracer>>,
}

impl Core_Systems<'_> {
    pub fn new(env: &Env_Info) -> Self {
        Core_Systems {
            input_system: input_system::Input_System::new(env),
            audio_system: audio_system::Audio_System::new(&audio_system::Audio_System_Config {
                max_concurrent_sounds: 10,
            }),
        }
    }
}

#[cfg(debug_assertions)]
impl Debug_Systems {
    pub fn new(cfg: &cfg::Config) -> Debug_Systems {
        Debug_Systems {
            debug_ui_system: debug_ui_system::Debug_Ui_System::new(),
            replay_recording_system: recording_system::Replay_Recording_System::new(
                recording_system::Replay_Recording_System_Config {
                    ms_per_frame: crate::cfg::Cfg_Var::<i32>::new(
                        "engine/gameplay/gameplay_update_tick_ms",
                        cfg,
                    )
                    .read(cfg),
                },
            ),
            tracer: Rc::new(RefCell::new(tracer::Debug_Tracer::new())),
        }
    }
}
