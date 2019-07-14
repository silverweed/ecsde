use super::msg;
use super::time::Time;
use std::time::Duration;

pub struct Time_Manager {
    pub time: Time,
}

impl Time_Manager {
    pub fn new() -> Time_Manager {
        Time_Manager { time: Time::new() }
    }
}

pub enum Time_Msg {
    Pause,
    Resume,
    Pause_Toggle,
    Set_Time_Scale(f32),
    Step(Duration),
    Get_Time_Scale,
    Is_Paused,
}

pub enum Time_Resp {
    Void,
    Cur_Time_Scale(f32),
    Is_Paused(bool),
}

impl msg::Msg_Responder for Time_Manager {
    type Msg_Data = Time_Msg;
    type Resp_Data = Time_Resp;

    fn send_message(&mut self, msg: Time_Msg) -> Time_Resp {
        match msg {
            Time_Msg::Pause => {
                self.time.set_paused(true);
                Time_Resp::Void
            }
            Time_Msg::Resume => {
                self.time.set_paused(false);
                Time_Resp::Void
            }
            Time_Msg::Pause_Toggle => {
                self.time.set_paused(!self.time.is_paused());
                Time_Resp::Void
            }
            Time_Msg::Set_Time_Scale(scale) => {
                self.time.set_time_scale(scale);
                Time_Resp::Void
            }
            Time_Msg::Step(dt) => {
                self.time.step(&dt);
                Time_Resp::Void
            }
            Time_Msg::Get_Time_Scale => Time_Resp::Cur_Time_Scale(self.time.get_time_scale()),
            Time_Msg::Is_Paused => Time_Resp::Is_Paused(self.time.is_paused()),
        }
    }
}
