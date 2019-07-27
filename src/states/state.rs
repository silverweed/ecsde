use crate::cfg;
use crate::core::world::World;
use crate::input::actions::Action_List;

pub enum State_Transition {
    None,
    Replace(Box<dyn Game_State>),
    Push(Box<dyn Game_State>),
    Pop,
}

pub trait Game_State {
    fn on_start(&mut self, _world: &World) {}
    fn on_pause(&mut self, _world: &World) {}
    fn on_resume(&mut self, _world: &World) {}
    fn on_end(&mut self, _world: &World) {}
    fn update(&mut self, _world: &World) -> State_Transition {
        State_Transition::None
    }
    /// Returns true if should quit
    fn handle_actions(
        &mut self,
        _actions: &Action_List,
        _world: &World,
        _config: &cfg::Config,
    ) -> bool {
        false
    }
}

pub trait Persistent_Game_State {
    fn on_start(&mut self, _world: &World) {}
    fn on_end(&mut self, _world: &World) {}
    fn update(&mut self, _world: &World) {}
    /// Returns true if should quit
    fn handle_actions(
        &mut self,
        _actions: &Action_List,
        _world: &World,
        _config: &cfg::Config,
    ) -> bool {
        false
    }
}
