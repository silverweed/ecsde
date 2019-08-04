use crate::core::common::stringid::String_Id;
use std::collections::{HashMap, HashSet};

/// A "Virtual Axis" is a user-defined axis with an arbitrary name.
/// A Virtual Axis can be mapped to any number of real joystick axes (e.g. Joystick_Axis::Stick_Left)
/// or to a set of Input_Actions: those can either set the axis value to max or to min (e.g.
/// Key::W may set the value to +1 and Key::S to -1).
#[derive(Clone, Default)]
pub struct Virtual_Axes {
    /// Map { virtual_axis_name => value [-1, 1] }
    pub(super) values: HashMap<String_Id, f32>,
    pub(super) value_comes_from_emulation: HashSet<String_Id>,
}

impl Virtual_Axes {
    pub fn with_axes(axes: &[String_Id]) -> Virtual_Axes {
        let mut values = HashMap::new();
        for name in axes.iter() {
            values.insert(*name, 0.0);
        }
        Virtual_Axes {
            values,
            value_comes_from_emulation: HashSet::new(),
        }
    }

    pub fn get_axis_value(&self, name: String_Id) -> f32 {
        if let Some(val) = self.values.get(&name) {
            *val
        } else {
            eprintln!("[ WARNING ] Queried value of inexistent axis {}", name);
            0.0
        }
    }

    pub(super) fn set_emulated_value(&mut self, name: String_Id, value: f32) {
        let val = self
            .values
            .get_mut(&name)
            .unwrap_or_else(|| panic!("Tried to set emulated value for inexistent axis {}", name));
        *val = value;
        self.value_comes_from_emulation.insert(name);
    }

    pub(super) fn reset_emulated_value(&mut self, name: String_Id) {
        self.value_comes_from_emulation.remove(&name);
    }
}
