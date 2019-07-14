use crate::audio;
use crate::core::input;
use crate::game::gameplay_system;
use crate::gfx;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Core_Systems {
    pub input_system: Rc<RefCell<input::Input_System>>,
    pub render_system: Rc<RefCell<gfx::render_system::Render_System>>,
    pub ui_system: Rc<RefCell<gfx::ui::UI_System>>,
    pub audio_system: Rc<RefCell<audio::system::Audio_System>>,
    pub gameplay_system: Rc<RefCell<gameplay_system::Gameplay_System>>,
}

impl Core_Systems {
    pub fn new() -> Core_Systems {
        Core_Systems {
            input_system: Rc::new(RefCell::new(input::Input_System::new())),
            render_system: Rc::new(RefCell::new(gfx::render_system::Render_System::new())),
            ui_system: Rc::new(RefCell::new(gfx::ui::UI_System::new())),
            audio_system: Rc::new(RefCell::new(audio::system::Audio_System::new(
                &audio::system::Audio_System_Config {
                    max_concurrent_sounds: 10,
                },
            ))),
            gameplay_system: Rc::new(RefCell::new(gameplay_system::Gameplay_System::new())),
        }
    }
}
