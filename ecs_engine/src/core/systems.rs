use crate::audio::audio_system;
use crate::collisions::collision_system;
use crate::common::stringid::String_Id;
use crate::gfx::render_system;
use std::collections::HashMap;

#[cfg(debug_assertions)]
use {
    crate::cfg,
    crate::debug::{console, debug_ui_system, painter::Debug_Painter, log},
    crate::replay::recording_system,
};

pub struct Core_Systems<'r> {
    pub audio_system: audio_system::Audio_System<'r>,
    pub collision_system: collision_system::Collision_System,
    pub render_system: render_system::Render_System,
}

#[cfg(debug_assertions)]
pub struct Debug_Systems {
    pub debug_ui_system: debug_ui_system::Debug_Ui_System,

    pub replay_recording_system: recording_system::Replay_Recording_System,
    // Note: we have one painter per level
    pub painters: HashMap<String_Id, Debug_Painter>,
    pub console: console::Console,
    pub log: log::Debug_Log,

    pub show_trace_overlay: bool,
    pub trace_overlay_update_t: f32,
}

impl Core_Systems<'_> {
    pub fn new() -> Self {
        Core_Systems {
            audio_system: audio_system::Audio_System::new(&audio_system::Audio_System_Config {
                max_concurrent_sounds: 10,
            }),
            collision_system: collision_system::Collision_System::new(),
            render_system: render_system::Render_System::new(),
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
                    ms_per_frame: crate::cfg::Cfg_Var::<f32>::new(
                        "engine/gameplay/gameplay_update_tick_ms",
                        cfg,
                    )
                    .read(cfg),
                },
            ),
            painters: HashMap::new(),
            show_trace_overlay: false,
            trace_overlay_update_t: 0.0,
            console: console::Console::new(),
            log: log::Debug_Log::with_hist_len(30 * 60),
        }
    }
}
