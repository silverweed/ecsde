use crate::gameplay_system::Gameplay_System;
use crate::states::state::Persistent_Game_State;
use ecs_engine::core::app::Engine_State;
use ecs_engine::core::common::stringid::String_Id;
use ecs_engine::input::input_system::{Action_Kind, Game_Action};

pub struct Game_Base_State {
    sid_quit: String_Id,
}

impl Game_Base_State {
    pub fn new() -> Game_Base_State {
        Game_Base_State {
            sid_quit: String_Id::from("quit"),
        }
    }
}

impl Persistent_Game_State for Game_Base_State {
    fn handle_actions(
        &mut self,
        actions: &[Game_Action],
        _state: &mut Engine_State,
        _gs: &mut Gameplay_System,
    ) -> bool {
        for action in actions.iter() {
            match action {
                (name, Action_Kind::Pressed) if *name == self.sid_quit => {
                    return true;
                }
                _ => (),
            }
        }

        false
    }
}
