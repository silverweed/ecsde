pub mod position_history_system;

pub struct Game_Debug_Systems {
    pub position_history_system: position_history_system::Position_History_System,
}

impl Default for Game_Debug_Systems {
    fn default() -> Self {
        Self {
            // @Incomplete: maybe allow setting the max hist size via cfg
            position_history_system: position_history_system::Position_History_System::new(256),
        }
    }
}
