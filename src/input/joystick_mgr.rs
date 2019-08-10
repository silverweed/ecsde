use super::bindings::joystick::{self, Joystick, Joystick_Axis};
use std::collections::HashMap;
use std::convert::TryInto;

pub type Real_Axes_Values = [f32; Joystick_Axis::_Count as usize];

pub struct Joystick_Manager {
    // @Speed: probably we can just save values in a vec
    joysticks: HashMap<Joystick, Real_Axes_Values>,
}

impl Joystick_Manager {
    pub fn new() -> Joystick_Manager {
        // @Temporary :joysticks:
        let mut joysticks = HashMap::new();
        joysticks.insert(
            Joystick {
                id: 0,
                joy_type: joystick::Joystick_Type::XBox360,
            },
            Real_Axes_Values::default(),
        );
        Joystick_Manager { joysticks }
    }

    pub fn update(&mut self) {
        for (joy, values) in self.joysticks.iter_mut() {
            Self::get_all_joy_axes_values(*joy, values);
        }
    }

    pub fn values(&self, joy: Joystick) -> Option<&Real_Axes_Values> {
        self.joysticks.get(&joy)
    }

    fn get_all_joy_axes_values(joystick: Joystick, values: &mut Real_Axes_Values) {
        for i in 0u8..Joystick_Axis::_Count as u8 {
            let axis = i
                .try_into()
                .unwrap_or_else(|_| panic!("Failed to convert {} to a valid Joystick_Axis!", i));
            values[i as usize] = joystick::get_joy_axis_value(joystick, axis);
        }
    }
}
