use super::bindings::joystick;
use super::input_system::Input_Raw_Event;
use super::joystick_mgr::Real_Axes_Values;
use super::provider::{Input_Provider, Input_Provider_Input};
use std::convert::TryInto;
use std::vec::Vec;

pub struct Default_Input_Provider {
    events: Vec<Input_Raw_Event>,
    axes: Real_Axes_Values,
}

impl Default_Input_Provider {
    pub fn new() -> Default_Input_Provider {
        Default_Input_Provider {
            events: vec![],
            axes: Real_Axes_Values::default(),
        }
    }
}

/// The default input provider just gets all events from the window
impl Input_Provider for Default_Input_Provider {
    fn update(&mut self, window: &mut Input_Provider_Input) {
        self.events.clear();
        while let Some(evt) = window.poll_event() {
            self.events.push(evt);
        }

        // @Incomplete :multiple_joysticks:
        let joystick = joystick::Joystick {
            id: 0,
            joy_type: joystick::Joystick_Type::XBox360,
        };
        for i in 0u8..joystick::Joystick_Axis::_Count as u8 {
            let axis = i
                .try_into()
                .unwrap_or_else(|_| panic!("Failed to convert {} to a valid Joystick_Axis!", i));
            self.axes[i as usize] = joystick::get_joy_axis_value(joystick, axis);
        }
    }

    fn get_events(&self) -> &[Input_Raw_Event] {
        &self.events
    }

    fn get_axes(&mut self, joystick: joystick::Joystick, axes: &mut Real_Axes_Values) {
        *axes = self.axes;
    }

    fn is_realtime_player_input(&self) -> bool {
        true
    }
}
