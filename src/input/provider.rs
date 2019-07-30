use std::vec::Vec;

#[cfg(feature = "use-sfml")]
pub type Input_Provider_Input = sfml::graphics::RenderWindow;

#[cfg(feature = "use-sfml")]
pub type Input_Provider_Output = sfml::window::Event;

/// An Input_Provider provides event data for the Input_System.
/// This can be e.g. the window event loop or some replay data.
pub trait Input_Provider {
    fn poll_events(&mut self, args: &mut Input_Provider_Input) -> Vec<Input_Provider_Output>;
    fn is_realtime_player_input(&self) -> bool;
}
