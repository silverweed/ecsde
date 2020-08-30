use super::state::{Game_State, Game_State_Args, State_Transition};
use inle_input::input_state::{Action_Kind, Game_Action};
use std::time::Duration;

#[derive(Default)]
pub struct In_Game_State {
    should_open_pause_menu: bool,
}

impl Game_State for In_Game_State {
    fn on_start(&mut self, args: &mut Game_State_Args) {
        args.gameplay_system.load_test_level(
            &mut args.engine_state,
            &mut args.game_resources,
            &mut args.level_batches,
        );
    }

    fn on_end(&mut self, args: &mut Game_State_Args) {
        args.gameplay_system.unload_test_level(args.engine_state);
    }

    fn update(
        &mut self,
        _args: &mut Game_State_Args,
        _dt: &Duration,
        _real_dt: &Duration,
    ) -> State_Transition {
        if self.should_open_pause_menu {
            self.should_open_pause_menu = false;
            return State_Transition::Push(Box::new(
                super::pause_menu_state::Pause_Menu_State::default(),
            ));
        }
        State_Transition::None
    }

    fn handle_actions(&mut self, actions: &[Game_Action], _args: &mut Game_State_Args) {
        for action in actions {
            match action {
                (name, Action_Kind::Pressed) if *name == sid!("open_pause_menu") => {
                    self.should_open_pause_menu = true;
                }
                _ => (),
            }
        }
    }
}
