use crate::gameplay_system::Gameplay_System;
use ecs_engine::core::app::Engine_State;
use ecs_engine::input::input_system::Game_Action;

pub enum State_Transition {
    None,
    Replace(Box<dyn Game_State>),
    Push(Box<dyn Game_State>),
    Pop,
}

pub trait Game_State {
    fn on_start(&mut self, _state: &mut Engine_State, _gs: &mut Gameplay_System) {}
    fn on_pause(&mut self, _state: &mut Engine_State, _gs: &mut Gameplay_System) {}
    fn on_resume(&mut self, _state: &mut Engine_State, _gs: &mut Gameplay_System) {}
    fn on_end(&mut self, _state: &mut Engine_State, _gs: &mut Gameplay_System) {}
    fn update(&mut self, _state: &mut Engine_State, _gs: &mut Gameplay_System) -> State_Transition {
        State_Transition::None
    }
    /// Returns true if should quit
    fn handle_actions(
        &mut self,
        _actions: &[Game_Action],
        _state: &mut Engine_State,
        _gs: &mut Gameplay_System,
    ) -> bool {
        false
    }
}

pub trait Persistent_Game_State {
    fn on_start(&mut self, _state: &mut Engine_State, _gs: &mut Gameplay_System) {}
    fn on_end(&mut self, _state: &mut Engine_State, _gs: &mut Gameplay_System) {}
    fn update(&mut self, _state: &mut Engine_State, _gs: &mut Gameplay_System) {}
    /// Returns true if should quit
    fn handle_actions(
        &mut self,
        _actions: &[Game_Action],
        _state: &mut Engine_State,
        _gs: &mut Gameplay_System,
    ) -> bool {
        false
    }
}
