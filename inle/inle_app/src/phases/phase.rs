use inle_common::stringid::String_Id;
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
    pub fn new(sid: String_Id) -> Self {
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

    fn handle_actions(&mut self, _actions: &[Game_Action], _args: &mut Self::Args) {}
}

pub trait Persistent_Game_Phase {
    type Args;

    fn on_start(&mut self, _args: &mut Self::Args) {}
    fn on_end(&mut self, _args: &mut Self::Args) {}
    fn update(&mut self, _args: &mut Self::Args) {}

    fn handle_actions(&mut self, _actions: &[Game_Action], _args: &mut Self::Args) {}
}
