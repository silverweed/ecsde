use crate::systems::interface::{Game_System, Update_Args};

pub mod position_history_system;

pub struct Game_Debug_Systems {
    systems: Vec<Box<dyn Game_System>>,
}

impl Game_Debug_Systems {
    pub fn new(cfg: &inle_cfg::Config) -> Self {
        Self {
            systems: vec![Box::new(
                position_history_system::Position_History_System::new(cfg),
            )],
        }
    }

    pub fn update(&self, args: &mut Update_Args) {
        for system in &self.systems {
            system.update(args);
        }
    }
}
