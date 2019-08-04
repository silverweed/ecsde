use super::bindings::joystick::Joystick_Axis;
use super::bindings::Input_Action;
use crate::core::common::stringid::String_Id;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub enum Virtual_Axis_Mapping {
    Axis(Joystick_Axis),
    Action_Emulate_Min(Input_Action),
    Action_Emulate_Max(Input_Action),
}

/// A "Virtual Axis" is a user-defined axis with an arbitrary name.
/// A Virtual Axis can be mapped to any number of real joystick axes (e.g. Joystick_Axis::Stick_Left)
/// or to a set of Input_Actions: those can either set the axis value to max or to min (e.g.
/// Key::W may set the value to +1 and Key::S to -1).
pub struct Virtual_Axes {
    /// Map virtual_axis_name => real_axes_or_keys
    axes_mappings: HashMap<String_Id, Vec<Virtual_Axis_Mapping>>,
}

impl Virtual_Axes {
    pub fn new() -> Virtual_Axes {
        Virtual_Axes {
            axes_mappings: HashMap::new(),
        }
    }

    pub fn register_virtual_axis(&mut self, name: &str) {
        match self.axes_mappings.entry(String_Id::from(name)) {
            Entry::Occupied(_) => eprintln!("[ NOTICE ] Tried to register axis {} twice!", name),
            Entry::Vacant(v) => {
                v.insert(vec![]);
            }
        }
    }
}
