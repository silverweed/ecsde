use super::env::Env_Info;
use crate::audio;
use crate::game::gameplay_system;
use crate::gfx;
use crate::input::input_system;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(debug_assertions)]
use crate::debug::debug_system;

pub struct Core_Systems {
    pub input_system: Rc<RefCell<input_system::Input_System>>,
    pub render_system: Rc<RefCell<gfx::render_system::Render_System>>,
    pub audio_system: Rc<RefCell<audio::system::Audio_System>>,
    pub gameplay_system: Rc<RefCell<gameplay_system::Gameplay_System>>,

    #[cfg(debug_assertions)]
    pub debug_system: Rc<RefCell<debug_system::Debug_System>>,
}

impl Core_Systems {
    pub fn new(env: &Env_Info) -> Core_Systems {
        Core_Systems {
            input_system: Rc::new(RefCell::new(input_system::Input_System::new(env))),
            render_system: Rc::new(RefCell::new(gfx::render_system::Render_System::new())),
            audio_system: Rc::new(RefCell::new(audio::system::Audio_System::new(
                &audio::system::Audio_System_Config {
                    max_concurrent_sounds: 10,
                },
            ))),
            gameplay_system: Rc::new(RefCell::new(gameplay_system::Gameplay_System::new())),
            #[cfg(debug_assertions)]
            debug_system: Rc::new(RefCell::new(debug_system::Debug_System::new())),
        }
    }
}
