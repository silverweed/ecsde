use crate::states::state::Persistent_Game_State;
use ecs_engine::cfg::{self, Cfg_Var};
use ecs_engine::core::app::Engine_State;
use ecs_engine::core::common::stringid::String_Id;
use ecs_engine::core::time;
use ecs_engine::input::input_system::{Action_Kind, Game_Action};
use std::convert::TryFrom;
use std::time::Duration;

pub struct Debug_Base_State {
    sid_game_speed_up: String_Id,
    sid_game_speed_down: String_Id,
    sid_pause_toggle: String_Id,
    sid_step_sim: String_Id,
    sid_print_em_debug_info: String_Id,
    sid_quit: String_Id,
    fps: Cfg_Var<i32>,
}

const CHANGE_SPEED_DELTA: f32 = 0.1;

impl Debug_Base_State {
    pub fn new(cfg: &cfg::Config) -> Debug_Base_State {
        Debug_Base_State {
            sid_game_speed_up: String_Id::from("game_speed_up"),
            sid_game_speed_down: String_Id::from("game_speed_down"),
            sid_pause_toggle: String_Id::from("pause_toggle"),
            sid_step_sim: String_Id::from("step_sim"),
            sid_print_em_debug_info: String_Id::from("print_em_debug_info"),
            sid_quit: String_Id::from("quit"),
            fps: Cfg_Var::new("engine/fps", cfg),
        }
    }
}

impl Persistent_Game_State for Debug_Base_State {
    fn handle_actions(&mut self, actions: &[Game_Action], state: &mut Engine_State) -> bool {
        let msg_overlay = state
            .debug_systems
            .debug_ui_system
            .get_fadeout_overlay(String_Id::from("msg"));

        // @Refactor: change the horrible if-else chain with a match.
        // This requires implementing a compile-time sid function (consider syntax extension).
        for action in actions.iter() {
            if (action.0 == self.sid_game_speed_up || action.0 == self.sid_game_speed_down)
                && action.1 == Action_Kind::Pressed
            {
                let ts = state.time.time_scale
                    + CHANGE_SPEED_DELTA
                        * if action.0 == self.sid_game_speed_up {
                            1.0
                        } else {
                            -1.0
                        };
                if ts > 0.0 {
                    state.time.time_scale = ts;
                }
                msg_overlay.add_line(&format!("Time scale: {:.2}", state.time.time_scale));
            } else if action.0 == self.sid_pause_toggle && action.1 == Action_Kind::Pressed {
                state.time.pause_toggle();
                msg_overlay.add_line(if state.time.paused {
                    "Paused"
                } else {
                    "Resumed"
                });
            } else if action.0 == self.sid_step_sim && action.1 == Action_Kind::Pressed {
                let target_fps = self.fps.read(&state.config);
                let step_delta =
                    Duration::from_nanos(u64::try_from(1_000_000_000 / target_fps).unwrap());
                msg_overlay.add_line(&format!(
                    "Stepping of: {:.2} ms",
                    time::to_secs_frac(&step_delta) * 1000.0
                ));
                state.time.paused = true;
                state.time.step(&step_delta);
            //state.systems.gameplay_system.step(&step_delta);
            } else if action.0 == self.sid_print_em_debug_info && action.1 == Action_Kind::Pressed {
                //state.systems.gameplay_system.print_debug_info();
            } else if action.0 == self.sid_quit && action.1 == Action_Kind::Pressed {
                return true;
            }
        }
        false
    }
}
