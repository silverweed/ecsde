use crate::joystick::Joystick_Id;

/// These actions are directly handled by the engine, without external configuration from data
#[non_exhaustive]
#[derive(Debug)]
pub enum Core_Action {
    Quit,
    Resize(u32, u32),
    Joystick_Connected { id: Joystick_Id },
    Joystick_Disconnected { id: Joystick_Id },
    Focus_Lost,
}
