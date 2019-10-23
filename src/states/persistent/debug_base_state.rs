use crate::cfg::Cfg_Var;
use crate::core::common::stringid::String_Id;
use crate::core::time;
use crate::core::world::World;
use crate::input::input_system::{Action_Kind, Game_Action};
use crate::states::state::Persistent_Game_State;
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
    pub fn new() -> Debug_Base_State {
        Debug_Base_State {
            sid_game_speed_up: String_Id::from("game_speed_up"),
            sid_game_speed_down: String_Id::from("game_speed_down"),
            sid_pause_toggle: String_Id::from("pause_toggle"),
            sid_step_sim: String_Id::from("step_sim"),
            sid_print_em_debug_info: String_Id::from("print_em_debug_info"),
            sid_quit: String_Id::from("quit"),
            fps: Cfg_Var::new("engine/fps"),
        }
    }
}

impl Persistent_Game_State for Debug_Base_State {
    fn handle_actions(&mut self, actions: &[Game_Action], world: &World) -> bool {
        let mut time = world.time.borrow_mut();
        let mut debug_system = world.get_systems().debug_system.borrow_mut();
        let msg_overlay = debug_system.get_fadeout_overlay(String_Id::from("msg"));

        // @Refactor: change the horrible if-else chain with a match.
        // This requires implementing a compile-time sid function (consider syntax extension).
        for action in actions.iter() {
            if (action.0 == self.sid_game_speed_up || action.0 == self.sid_game_speed_down)
                && action.1 == Action_Kind::Pressed
            {
                let ts = time.get_time_scale()
                    + CHANGE_SPEED_DELTA
                        * if action.0 == self.sid_game_speed_up {
                            1.0
                        } else {
                            -1.0
                        };
                if ts > 0.0 {
                    time.set_time_scale(ts);
                }
                msg_overlay.add_line(&format!("Time scale: {:.2}", time.get_time_scale()));
            } else if action.0 == self.sid_pause_toggle && action.1 == Action_Kind::Pressed {
                let paused = time.is_paused();
                time.set_paused(!paused);
                msg_overlay.add_line(if !paused { "Paused" } else { "Resumed" });
            } else if action.0 == self.sid_step_sim && action.1 == Action_Kind::Pressed {
                let target_fps = self.fps.read();
                let step_delta =
                    Duration::from_nanos(u64::try_from(1_000_000_000 / target_fps).unwrap());
                msg_overlay.add_line(&format!(
                    "Stepping of: {:.2} ms",
                    time::to_secs_frac(&step_delta) * 1000.0
                ));
                time.set_paused(true);
                time.step(&step_delta);
                let mut gameplay_system = world.get_systems().gameplay_system.borrow_mut();
                gameplay_system.step(&step_delta);
            } else if action.0 == self.sid_print_em_debug_info && action.1 == Action_Kind::Pressed {
                let gameplay_system = world.get_systems().gameplay_system.borrow();
                gameplay_system.print_debug_info();
            } else if action.0 == self.sid_quit && action.1 == Action_Kind::Pressed {
                return true;
            }
        }
        false
    }
}
