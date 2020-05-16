use crate::game_state::{Game_Resources, Level_Batches};
use crate::gameplay_system::Gameplay_System;
use ecs_engine::core::app::Engine_State;
use ecs_engine::gfx::window::Window_Handle;
use ecs_engine::input::input_system::Game_Action;
use std::time::Duration;

pub enum State_Transition {
    None,
    Replace(Box<dyn Game_State>), // replaces the topmost state
    Flush_All_And_Replace(Box<dyn Game_State>),
    Push(Box<dyn Game_State>),
    Pop,
}

pub struct Game_State_Args<'e, 'r, 'g, 'w, 'r1, 'r2> {
    pub engine_state: &'e mut Engine_State<'r>,
    pub gameplay_system: &'g mut Gameplay_System,
    pub window: &'w mut Window_Handle,
    pub game_resources: &'r1 mut Game_Resources<'r2>,
    pub level_batches: &'g mut Level_Batches,
}

pub trait Game_State {
    fn on_start(&mut self, _args: &mut Game_State_Args) {}
    fn on_pause(&mut self, _args: &mut Game_State_Args) {}
    fn on_resume(&mut self, _args: &mut Game_State_Args) {}
    fn on_end(&mut self, _args: &mut Game_State_Args) {}
    fn update(
        &mut self,
        _args: &mut Game_State_Args,
        _dt: &Duration,
        _real_dt: &Duration,
    ) -> State_Transition {
        State_Transition::None
    }
    fn handle_actions(&mut self, _actions: &[Game_Action], _args: &mut Game_State_Args) {}
}

pub trait Persistent_Game_State {
    fn on_start(&mut self, _args: &mut Game_State_Args) {}
    fn on_end(&mut self, _args: &mut Game_State_Args) {}
    fn update(&mut self, _args: &mut Game_State_Args, _dt: &Duration, _real_dt: &Duration) {}
    /// Returns true if should quit
    fn handle_actions(&mut self, _actions: &[Game_Action], _args: &mut Game_State_Args) {}
}
