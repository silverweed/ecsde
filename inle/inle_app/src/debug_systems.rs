use std::sync::{Arc, Mutex};
use {
    inle_core::env::Env_Info,
    inle_common::stringid::String_Id,
    inle_core::rand::Default_Rng_Seed,
    inle_debug::{calipers, console, debug_ui, log, painter::Debug_Painter},
    inle_resources::gfx::Gfx_Resources,
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Overlay_Shown {
    None,
    Trace,
    Threads,
}

pub struct Debug_Systems {
    pub debug_ui: debug_ui::Debug_Ui_System,

    pub global_painter: Debug_Painter,
    pub console: Arc<Mutex<console::Console>>,
    pub log: log::Debug_Log,
    pub calipers: calipers::Debug_Calipers,

    pub show_overlay: Overlay_Shown,
    pub trace_overlay_update_t: f32,
    pub traced_fn: String,
}

impl Debug_Systems {
    pub fn new(cfg: &inle_cfg::Config, rng_seed: Default_Rng_Seed) -> Debug_Systems {
        let ms_per_frame =
            inle_cfg::Cfg_Var::<f32>::new("engine/gameplay/update_tick_ms", cfg).read(cfg);
        let debug_log_size =
            inle_cfg::Cfg_Var::<i32>::new("engine/debug/log/hist_size_seconds", cfg).read(cfg);
        let fps = (1000. / ms_per_frame + 0.5) as i32;
        Debug_Systems {
            debug_ui: debug_ui::Debug_Ui_System::default(),
            global_painter: Debug_Painter::default(),
            show_overlay: Overlay_Shown::None,
            trace_overlay_update_t: 0.0,
            console: Arc::new(Mutex::new(console::Console::new())),
            log: log::Debug_Log::with_hist_len((debug_log_size * fps) as _),
            calipers: calipers::Debug_Calipers::default(),
            traced_fn: String::default(),
        }
    }
}
