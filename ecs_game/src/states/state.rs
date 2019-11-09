use crate::core::app::Engine_State;
use crate::input::input_system::Game_Action;

pub enum State_Transition {
    None,
    Replace(Box<dyn Game_State>),
    Push(Box<dyn Game_State>),
    Pop,
}

pub trait Game_State {
    fn on_start(&mut self, _state: &mut Engine_State) {}
    fn on_pause(&mut self, _state: &mut Engine_State) {}
    fn on_resume(&mut self, _state: &mut Engine_State) {}
    fn on_end(&mut self, _state: &mut Engine_State) {}
    fn update(&mut self, _state: &mut Engine_State) -> State_Transition {
        State_Transition::None
    }
    /// Returns true if should quit
    fn handle_actions(&mut self, _actions: &[Game_Action], _state: &mut Engine_State) -> bool {
        false
    }
}

pub trait Persistent_Game_State {
    fn on_start(&mut self, _state: &mut Engine_State) {}
    fn on_end(&mut self, _state: &mut Engine_State) {}
    fn update(&mut self, _state: &mut Engine_State) {}
    /// Returns true if should quit
    fn handle_actions(&mut self, _actions: &[Game_Action], _state: &mut Engine_State) -> bool {
        false
    }
}
