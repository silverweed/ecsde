use super::bindings::joystick;
use super::input_system::Input_Raw_Event;
use super::joystick_state::{Joystick_State, Real_Axes_Values};
use crate::cfg;

#[cfg(feature = "use-sfml")]
pub type Input_Provider_Input = sfml::graphics::RenderWindow;

/// An Input_Provider provides event data for the Input_System.
/// This can be e.g. the window event loop or some replay data.
pub trait Input_Provider {
    fn update(
        &mut self,
        args: &mut Input_Provider_Input,
        joy_mgr: Option<&Joystick_State>,
        cfg: &cfg::Config,
    );
    fn get_events(&self) -> &[Input_Raw_Event];
    fn get_axes(&self, axes: &mut [Real_Axes_Values; joystick::JOY_COUNT as usize]);
    fn is_realtime_player_input(&self) -> bool;
}
