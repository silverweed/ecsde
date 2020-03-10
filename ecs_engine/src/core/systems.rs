use crate::audio::audio_system;
use crate::collisions::collision_system;
use crate::gfx::render_system;

#[cfg(debug_assertions)]
use {
    crate::cfg,
    crate::resources::gfx::Gfx_Resources,
    crate::core::env::Env_Info,
    crate::common::stringid::String_Id,
    crate::debug::{console, debug_ui_system, log, painter::Debug_Painter},
    crate::replay::recording_system,
    std::collections::HashMap,
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
    // Note: we have one painter per level plus the "global" painter (with empty Id)
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
        let mut painters = HashMap::new();
        let ms_per_frame =
            cfg::Cfg_Var::<f32>::new("engine/gameplay/gameplay_update_tick_ms", cfg).read(cfg);
        let debug_log_size =
            cfg::Cfg_Var::<i32>::new("engine/debug/log/hist_size_seconds", cfg).read(cfg);
        let fps = (1000. / ms_per_frame + 0.5) as i32;
        painters.insert(String_Id::from(""), Debug_Painter::new());
        Debug_Systems {
            debug_ui_system: debug_ui_system::Debug_Ui_System::new(),
            replay_recording_system: recording_system::Replay_Recording_System::new(
                recording_system::Replay_Recording_System_Config { ms_per_frame },
            ),
            painters,
            show_trace_overlay: false,
            trace_overlay_update_t: 0.0,
            console: console::Console::new(),
            log: log::Debug_Log::with_hist_len((debug_log_size * fps) as _),
        }
    }

    pub fn global_painter(&mut self) -> &mut Debug_Painter {
        self.painters.get_mut(&String_Id::from("")).unwrap()
    }

    pub fn new_debug_painter_for_level(&mut self, lvid: String_Id, gres: &mut Gfx_Resources, env: &Env_Info) {
        assert!(!self.painters.contains_key(&lvid), "Multiple painters added for level {}", lvid);

        let mut painter = Debug_Painter::new();
        painter.init(gres, env);
        self.painters.insert(lvid, painter);
    }
}
