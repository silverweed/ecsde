use super::bindings::Axis_Emulation_Type;
use crate::core::common::stringid::String_Id;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

/// A "Virtual Axis" is a user-defined axis with an arbitrary name.
/// A Virtual Axis can be mapped to any number of real joystick axes (e.g. Joystick_Axis::Stick_Left)
/// or to a set of Input_Actions: those can either set the axis value to max or to min (e.g.
/// Key::W may set the value to +1 and Key::S to -1).
#[derive(Clone, Default, Debug)]
pub struct Virtual_Axes {
    /// Map { virtual_axis_name => value [-1, 1] }
    pub(super) values: HashMap<String_Id, f32>,
    /// Map { virtual_axis_name => (emulated_min_is_pressed, emulated_max_is_pressed)
    pub(super) value_comes_from_emulation: HashMap<String_Id, (bool, bool)>,
}

impl Virtual_Axes {
    pub fn with_axes(axes: &[String_Id]) -> Virtual_Axes {
        let mut values = HashMap::new();
        for name in axes.iter() {
            values.insert(*name, 0.0);
        }
        Virtual_Axes {
            values,
            value_comes_from_emulation: HashMap::new(),
        }
    }

    pub fn get_all_values(&self) -> HashMap<String_Id, f32> {
        self.values.clone()
    }

    pub fn get_axis_value(&self, name: String_Id) -> f32 {
        if let Some(val) = self.values.get(&name) {
            *val
        } else {
            eprintln!("[ WARNING ] Queried value of inexistent axis {}", name);
            0.0
        }
    }

    pub(super) fn set_emulated_value(&mut self, name: String_Id, emu_kind: Axis_Emulation_Type) {
        let val = self
            .values
            .get_mut(&name)
            .unwrap_or_else(|| panic!("Tried to set emulated value for inexistent axis {}", name));

        *val = emu_kind.assoc_value();

        match self.value_comes_from_emulation.entry(name) {
            Entry::Occupied(mut o) => {
                let (min, max) = o.get_mut();
                if emu_kind == Axis_Emulation_Type::Min {
                    *min = true;
                } else {
                    *max = true;
                }
            }
            Entry::Vacant(v) => {
                v.insert((
                    emu_kind == Axis_Emulation_Type::Min,
                    emu_kind == Axis_Emulation_Type::Max,
                ));
            }
        }
    }

    pub(super) fn reset_emulated_value(&mut self, name: String_Id, emu_kind: Axis_Emulation_Type) {
        if let Some((min, max)) = self.value_comes_from_emulation.get_mut(&name) {
            match emu_kind {
                Axis_Emulation_Type::Min => {
                    *min = false;
                    if *max {
                        *self.values.get_mut(&name).unwrap() =
                            Axis_Emulation_Type::Max.assoc_value();
                    }
                }
                Axis_Emulation_Type::Max => {
                    *max = false;
                    if *min {
                        *self.values.get_mut(&name).unwrap() =
                            Axis_Emulation_Type::Min.assoc_value();
                    }
                }
            }
        }
    }
}
