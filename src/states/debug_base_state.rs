use super::state::Persistent_Game_State;
use crate::cfg;
use crate::core::msg::Msg_Responder;
use crate::core::time;
use crate::core::time_manager::{Time_Manager, Time_Msg, Time_Resp};
use crate::core::world::World;
use crate::game::gameplay_system::{Gameplay_System, Gameplay_System_Msg};
use crate::gfx::ui::{UI_Request, UI_System};
use crate::input::input_system::{Action, Action_List};
use std::convert::TryFrom;
use std::time::Duration;

pub struct Debug_Base_State {}

impl Persistent_Game_State for Debug_Base_State {
    fn handle_actions(
        &mut self,
        actions: &Action_List,
        world: &World,
        config: &cfg::Config,
    ) -> bool {
        if actions.has_action(&Action::Quit) {
            true
        } else {
            let dispatcher = world.get_dispatcher();
            let mut time_mgr = dispatcher.borrow_mut::<Time_Manager>().unwrap();
            let mut ui_system = dispatcher.borrow_mut::<UI_System>().unwrap();
            for action in actions.iter() {
                match action {
                    Action::Change_Speed(delta) => {
                        let ts = if let Time_Resp::Cur_Time_Scale(ts) =
                            time_mgr.send_message(Time_Msg::Get_Time_Scale)
                        {
                            ts + *delta as f32 * 0.01
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
                    }
                    Action::Pause_Toggle => {
                        time_mgr.send_message(Time_Msg::Pause_Toggle);
                        let paused = if let Time_Resp::Is_Paused(paused) =
                            time_mgr.send_message(Time_Msg::Is_Paused)
                        {
                            paused
                        } else {
                            panic!("[ FATAL ] unexpected response from time_mgr!");
                        };
                        ui_system.send_message(UI_Request::Add_Fadeout_Text(String::from(
                            if paused { "Paused" } else { "Resumed" },
                        )));
                    }
                    Action::Step_Simulation => {
                        let target_fps = config.get_var_int_or("engine/rendering/fps", 60);
                        let step_delta = Duration::from_nanos(
                            u64::try_from(1_000_000_000 / *target_fps).unwrap(),
                        );
                        ui_system.send_message(UI_Request::Add_Fadeout_Text(format!(
                            "Stepping of: {:.2} ms",
                            time::to_secs_frac(&step_delta) * 1000.0
                        )));
                        time_mgr.send_message(Time_Msg::Pause);
                        time_mgr.send_message(Time_Msg::Step(step_delta));
                        dispatcher
                            .send_message::<Gameplay_System>(Gameplay_System_Msg::Step(step_delta));
                    }
                    Action::Print_Entity_Manager_Debug_Info => {
                        dispatcher.send_message::<Gameplay_System>(
                            Gameplay_System_Msg::Print_Entity_Manager_Debug_Info,
                        );
                    }
                    _ => (),
                }
            }
            false
        }
    }
}
