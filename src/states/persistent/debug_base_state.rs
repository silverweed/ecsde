use crate::cfg;
use crate::core::common::stringid::String_Id;
use crate::core::msg::Msg_Responder;
use crate::core::time;
use crate::core::time_manager::{Time_Manager, Time_Msg, Time_Resp};
use crate::core::world::World;
use crate::game::gameplay_system::{Gameplay_System, Gameplay_System_Msg};
use crate::gfx::ui::{UI_Request, UI_System};
use crate::input::input_system::Game_Action;
use crate::states::state::Persistent_Game_State;
use std::convert::TryFrom;
use std::time::Duration;

pub struct Debug_Base_State {
    sid_change_speed: String_Id,
    sid_pause_toggle: String_Id,
    sid_step_sim: String_Id,
    sid_print_em_debug_info: String_Id,
    sid_quit: String_Id,
}

const CHANGE_SPEED_DELTA: f32 = 0.1;

impl Debug_Base_State {
    pub fn new() -> Debug_Base_State {
        Debug_Base_State {
            sid_change_speed: String_Id::from("change_speed"),
            sid_pause_toggle: String_Id::from("pause_toggle"),
            sid_step_sim: String_Id::from("step_sim"),
            sid_print_em_debug_info: String_Id::from("print_em_debug_info"),
            sid_quit: String_Id::from("quit"),
        }
    }
}

impl Persistent_Game_State for Debug_Base_State {
    fn handle_actions(
        &mut self,
        actions: &[Game_Action],
        world: &World,
        config: &cfg::Config,
    ) -> bool {
        let dispatcher = world.get_dispatcher();
        let mut time_mgr = dispatcher.borrow_mut::<Time_Manager>().unwrap();
        let mut ui_system = dispatcher.borrow_mut::<UI_System>().unwrap();

        for action in actions.iter() {
            if action.0 == self.sid_change_speed {
                let ts = if let Time_Resp::Cur_Time_Scale(ts) =
                    time_mgr.send_message(Time_Msg::Get_Time_Scale)
                {
                    ts + CHANGE_SPEED_DELTA
                } else {
                    panic!("[ FATAL ] unexpected response from time_mgr!");
                };
                if ts > 0.0 {
                    time_mgr.send_message(Time_Msg::Set_Time_Scale(ts));
                }
                let ts = if let Time_Resp::Cur_Time_Scale(ts) =
                    time_mgr.send_message(Time_Msg::Get_Time_Scale)
                {
                    ts
                } else {
                    panic!("[ FATAL ] unexpected response from time_mgr!");
                };
                ui_system.send_message(UI_Request::Add_Fadeout_Text(format!(
                    "Time scale: {:.2}",
                    ts
                )));
            } else if action.0 == self.sid_pause_toggle {
                time_mgr.send_message(Time_Msg::Pause_Toggle);
                let paused = if let Time_Resp::Is_Paused(paused) =
                    time_mgr.send_message(Time_Msg::Is_Paused)
                {
                    paused
                } else {
                    panic!("[ FATAL ] unexpected response from time_mgr!");
                };
                ui_system.send_message(UI_Request::Add_Fadeout_Text(String::from(if paused {
                    "Paused"
                } else {
                    "Resumed"
                })));
            } else if action.0 == self.sid_step_sim {
                let target_fps = config.get_var_int_or("engine/rendering/fps", 60);
                let step_delta =
                    Duration::from_nanos(u64::try_from(1_000_000_000 / *target_fps).unwrap());
                ui_system.send_message(UI_Request::Add_Fadeout_Text(format!(
                    "Stepping of: {:.2} ms",
                    time::to_secs_frac(&step_delta) * 1000.0
                )));
                time_mgr.send_message(Time_Msg::Pause);
                time_mgr.send_message(Time_Msg::Step(step_delta));
                dispatcher.send_message::<Gameplay_System>(Gameplay_System_Msg::Step(step_delta));
            } else if action.0 == self.sid_print_em_debug_info {
                dispatcher.send_message::<Gameplay_System>(
                    Gameplay_System_Msg::Print_Entity_Manager_Debug_Info,
                );
            } else if action.0 == self.sid_quit {
                return true;
            }
        }
        false
    }
}
