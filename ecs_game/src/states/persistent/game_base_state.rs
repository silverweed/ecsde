use crate::gameplay_system::Gameplay_System;
use crate::states::state::Persistent_Game_State;
use ecs_engine::core::app::Engine_State;
use ecs_engine::input::input_system::Game_Action;

pub struct Game_Base_State {}

impl Persistent_Game_State for Game_Base_State {
    fn handle_actions(
        &mut self,
        _actions: &[Game_Action],
        _state: &mut Engine_State,
        _gs: &mut Gameplay_System,
    ) -> bool {
        false
    }
}
