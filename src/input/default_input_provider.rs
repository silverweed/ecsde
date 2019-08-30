use super::bindings::joystick;
use super::input_system::Input_Raw_Event;
use super::joystick_mgr::{Joystick_Manager, Real_Axes_Values};
use super::provider::{Input_Provider, Input_Provider_Input};
use std::convert::TryInto;
use std::vec::Vec;

#[derive(Default)]
pub struct Default_Input_Provider {
    pub events: Vec<Input_Raw_Event>,
    pub axes: [Real_Axes_Values; joystick::JOY_COUNT as usize],
}

/// The default input provider just gets all events from the window
impl Input_Provider for Default_Input_Provider {
    fn update(&mut self, window: &mut Input_Provider_Input, joy_mgr: &Joystick_Manager) {
        self.events.clear();
        while let Some(evt) = window.poll_event() {
            self.events.push(evt);
        }

        for joy_id in 0..joystick::JOY_COUNT {
            if let Some(joy) = joy_mgr.get_joystick(joy_id) {
                for i in 0u8..joystick::Joystick_Axis::_Count as u8 {
                    let axis = i.try_into().unwrap_or_else(|err| {
                        panic!("Failed to convert {} to a valid Joystick_Axis: {}", i, err)
                    });
                    self.axes[joy_id as usize][i as usize] =
                        joystick::get_joy_axis_value(*joy, axis);
                }
            }
        }
    }

    fn get_events(&self) -> &[Input_Raw_Event] {
        &self.events
    }

    fn get_axes(&self, axes: &mut [Real_Axes_Values; joystick::JOY_COUNT as usize]) {
        *axes = self.axes;
    }

    fn is_realtime_player_input(&self) -> bool {
        true
    }
}
