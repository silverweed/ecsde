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

    pub fn values(&self, joy: Joystick) -> Option<&Real_Axes_Values> {
        self.joysticks.get(&joy)
    }

    pub(super) fn mut_values(&mut self, joy: Joystick) -> Option<&mut Real_Axes_Values> {
        self.joysticks.get_mut(&joy)
    }
}
