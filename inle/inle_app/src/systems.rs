use inle_audio::audio_system;
use inle_common::stringid::String_Id;
use inle_core::tasks::Long_Task_Manager;
use inle_events::evt_register;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
    pub long_task_mgr: Long_Task_Manager,
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
            long_task_mgr: Long_Task_Manager::default(),
        }
    }
}

#[cfg(debug_assertions)]
