use super::bindings::joystick::Joystick;
use super::input_system::Input_Raw_Event;
use super::joystick_mgr::Real_Axes_Values;

#[cfg(feature = "use-sfml")]
pub type Input_Provider_Input = sfml::graphics::RenderWindow;

/// An Input_Provider provides event data for the Input_System.
/// This can be e.g. the window event loop or some replay data.
pub trait Input_Provider {
    fn update(&mut self, args: &mut Input_Provider_Input);
    fn get_events(&self) -> &[Input_Raw_Event];
    fn get_axes(&mut self, joystick: Joystick, axes: &mut Real_Axes_Values);
    fn is_realtime_player_input(&self) -> bool;
}
