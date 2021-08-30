use inle_cfg::Cfg_Var;

pub mod position_history_system;

pub struct Game_Debug_Systems {
    pub position_history_system: position_history_system::Position_History_System,
}

impl Game_Debug_Systems {
    pub fn new(cfg: &inle_cfg::Config) -> Self {
        Self {
            // @Incomplete: maybe allow setting the max hist size via cfg
            position_history_system: position_history_system::Position_History_System::new(
                Cfg_Var::new("debug/entities/pos_history/hist_size", cfg),
            ),
        }
    }
}
