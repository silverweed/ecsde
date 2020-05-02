use crate::game_state::Game_Resources;
use crate::gameplay_system::Gameplay_System;
use ecs_engine::core::app::Engine_State;
use ecs_engine::gfx::window::Window_Handle;
use ecs_engine::input::input_system::Game_Action;

pub enum State_Transition {
    None,
    Replace(Box<dyn Game_State>),
    Push(Box<dyn Game_State>),
    Pop,
}

pub struct Game_State_Args<'e, 'r, 'g, 'w, 'r1, 'r2> {
    pub engine_state: &'e mut Engine_State<'r>,
    pub gameplay_system: &'g mut Gameplay_System,
    pub window: &'w mut Window_Handle,
    pub game_resources: &'r1 mut Game_Resources<'r2>,
}

pub trait Game_State {
    fn on_start(&mut self, _args: &mut Game_State_Args) {}
    fn on_pause(&mut self, _args: &mut Game_State_Args) {}
    fn on_resume(&mut self, _args: &mut Game_State_Args) {}
    fn on_end(&mut self, _args: &mut Game_State_Args) {}
    fn update(&mut self, _args: &mut Game_State_Args) -> State_Transition {
        State_Transition::None
    }
    /// Returns true if should quit
    fn handle_actions(&mut self, _actions: &[Game_Action], _args: &mut Game_State_Args) -> bool {
        false
    }
}

pub trait Persistent_Game_State {
    fn on_start(&mut self, _args: &mut Game_State_Args) {}
    fn on_end(&mut self, _args: &mut Game_State_Args) {}
    fn update(&mut self, _args: &mut Game_State_Args) {}
    /// Returns true if should quit
    fn handle_actions(&mut self, _actions: &[Game_Action], _args: &mut Game_State_Args) -> bool {
        false
    }
}
