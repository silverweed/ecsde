use inle_audio::audio_system;
use inle_common::stringid::String_Id;
use inle_events::evt_register;
use std::collections::HashMap;

#[cfg(debug_assertions)]
use {
    inle_core::env::Env_Info,
    inle_core::rand::Default_Rng_Seed,
    inle_debug::{calipers, console, debug_ui, log, painter::Debug_Painter},
    inle_replay::recording_system,
    inle_resources::gfx::Gfx_Resources,
};

pub struct Core_Systems<'r> {
    pub audio_system: audio_system::Audio_System<'r>,
    pub evt_register: evt_register::Event_Register,
    pub ui: inle_ui::Ui_Context,
    pub physics_settings: inle_physics::physics::Physics_Settings,
    // One particle manager per level
    pub particle_mgrs: HashMap<String_Id, inle_gfx::particles::Particle_Manager>,
}

#[cfg(debug_assertions)]
pub struct Debug_Systems {
    pub debug_ui: debug_ui::Debug_Ui_System,

    pub replay_recording_system: recording_system::Replay_Recording_System,
    // Note: we have one painter per level
    pub painters: HashMap<String_Id, Debug_Painter>,
    pub global_painter: Debug_Painter,
    pub console: console::Console,
    pub log: log::Debug_Log,
    pub calipers: calipers::Debug_Calipers,

    pub show_trace_overlay: bool,
    pub trace_overlay_update_t: f32,
    pub traced_fn: String,
}

impl Core_Systems<'_> {
    pub fn new() -> Self {
        Core_Systems {
            audio_system: audio_system::Audio_System::new(&audio_system::Audio_System_Config {
                max_concurrent_sounds: 10,
            }),
            evt_register: evt_register::Event_Register::new(),
            ui: inle_ui::Ui_Context::default(),
            physics_settings: inle_physics::physics::Physics_Settings::default(),
            particle_mgrs: HashMap::new(),
        }
    }
}

#[cfg(debug_assertions)]
impl Debug_Systems {
    pub fn new(cfg: &inle_cfg::Config, rng_seed: Default_Rng_Seed) -> Debug_Systems {
        let ms_per_frame =
            inle_cfg::Cfg_Var::<f32>::new("engine/gameplay/gameplay_update_tick_ms", cfg).read(cfg);
        let debug_log_size =
            inle_cfg::Cfg_Var::<i32>::new("engine/debug/log/hist_size_seconds", cfg).read(cfg);
        let fps = (1000. / ms_per_frame + 0.5) as i32;
        Debug_Systems {
            debug_ui: debug_ui::Debug_Ui_System::new(),
            replay_recording_system: recording_system::Replay_Recording_System::new(
                recording_system::Replay_Recording_System_Config {
                    ms_per_frame,
                    rng_seed,
                },
            ),
            painters: HashMap::default(),
            global_painter: Debug_Painter::new(),
            show_trace_overlay: false,
            trace_overlay_update_t: 0.0,
            console: console::Console::new(),
            log: log::Debug_Log::with_hist_len((debug_log_size * fps) as _),
            calipers: calipers::Debug_Calipers::default(),
            traced_fn: String::default(),
        }
    }

    pub fn new_debug_painter_for_level(
        &mut self,
        lvid: String_Id,
        gres: &mut Gfx_Resources,
        env: &Env_Info,
    ) {
        assert!(
            !self.painters.contains_key(&lvid),
            "Multiple painters added for level {}",
            lvid
        );

        let mut painter = Debug_Painter::new();
        painter.init(gres, env);
        self.painters.insert(lvid, painter);
    }
}
