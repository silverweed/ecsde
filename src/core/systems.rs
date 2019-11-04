use super::env::Env_Info;
use crate::audio;
use crate::game::gameplay_system;
use crate::gfx;
use crate::input::input_system;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(debug_assertions)]
use crate::debug::debug_ui_system;
#[cfg(debug_assertions)]
use crate::replay::recording_system;

pub struct Core_Systems {
    pub input_system: input_system::Input_System,
    pub render_system: gfx::render_system::Render_System,
    pub audio_system: audio::system::Audio_System,
    pub gameplay_system: gameplay_system::Gameplay_System,
}

#[cfg(debug_assertions)]
pub struct Debug_Systems {
    pub debug_ui_system: debug_ui_system::Debug_Ui_System,
    pub replay_recording_system: recording_system::Replay_Recording_System,
}

impl Core_Systems {
    pub fn new(env: &Env_Info) -> Core_Systems {
        Core_Systems {
            input_system: input_system::Input_System::new(env),
            render_system: gfx::render_system::Render_System::new(),
            audio_system: audio::system::Audio_System::new(&audio::system::Audio_System_Config {
                max_concurrent_sounds: 10,
            }),
            gameplay_system: gameplay_system::Gameplay_System::new(),
        }
    }
}

#[cfg(debug_assertions)]
impl Debug_Systems {
    pub fn new() -> Debug_Systems {
        Debug_Systems {
            debug_ui_system: debug_ui_system::Debug_Ui_System::new(),
            replay_recording_system: recording_system::Replay_Recording_System::new(
                recording_system::Replay_Recording_System_Config {
                    ms_per_frame: crate::cfg::Cfg_Var::new(
                        "engine/gameplay/gameplay_update_tick_ms",
                    ),
                },
            ),
        }
    }
}
