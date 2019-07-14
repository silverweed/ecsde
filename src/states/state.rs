use crate::cfg;
use crate::core::input::Action_List;
use crate::core::msg;
use std::time::Duration;

pub enum State_Transition {
    None,
    Replace(Box<dyn Game_State>),
    Push(Box<dyn Game_State>),
    Pop,
}

pub trait Game_State {
    fn on_start(&mut self) {}
    fn on_pause(&mut self) {}
    fn on_resume(&mut self) {}
    fn on_end(&mut self) {}
    fn update(&mut self, _dt: &Duration) -> State_Transition {
        State_Transition::None
    }
    /// Returns true if should quit
    fn handle_actions(
        &mut self,
        _actions: &Action_List,
        _dispatcher: &msg::Msg_Dispatcher,
        _config: &cfg::Config,
    ) -> bool {
        false
    }
}

pub trait Persistent_Game_State {
    fn on_start(&mut self) {}
    fn on_end(&mut self) {}
    fn update(&mut self, _dt: &Duration) {}
    /// Returns true if should quit
    fn handle_actions(
        &mut self,
        _actions: &Action_List,
        _dispatcher: &msg::Msg_Dispatcher,
        _config: &cfg::Config,
    ) -> bool {
        false
    }
}
