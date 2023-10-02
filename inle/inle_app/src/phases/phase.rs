use inle_common::stringid::{self, String_Id};
use inle_input::input_state::Game_Action;

pub enum Phase_Transition {
    None,
    Replace(Phase_Id), // replaces the topmost state
    Flush_All_And_Replace(Phase_Id),
    Push(Phase_Id),
    Pop,
    Quit_Game,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Phase_Id(String_Id);

impl Phase_Id {
    pub const fn new(s: &str) -> Self {
        Self(stringid::const_sid_from_str(s))
    }
}

impl From<String_Id> for Phase_Id {
    fn from(sid: String_Id) -> Self {
        Self(sid)
    }
}

pub trait Game_Phase {
    type Args;

    fn on_start(&mut self, _args: &mut Self::Args) {}
    fn on_pause(&mut self, _args: &mut Self::Args) {}
    fn on_resume(&mut self, _args: &mut Self::Args) {}
    fn on_end(&mut self, _args: &mut Self::Args) {}

    fn update(&mut self, _args: &mut Self::Args) -> Phase_Transition;
    fn draw(&self, _args: &mut Self::Args) {}

    fn handle_actions(&mut self, _actions: &[Game_Action], _args: &mut Self::Args) {}
}

pub trait Persistent_Game_Phase {
    type Args;

    fn on_start(&mut self, _args: &mut Self::Args) {}
    fn on_end(&mut self, _args: &mut Self::Args) {}
    fn update(&mut self, _args: &mut Self::Args) {}
    fn draw(&self, _args: &mut Self::Args) {}

    fn handle_actions(&mut self, _actions: &[Game_Action], _args: &mut Self::Args) {}
}
