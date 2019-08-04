use super::axes::Virtual_Axis_Mapping;
use crate::core::common::stringid::String_Id;
use std::collections::HashMap;
use std::path::Path;
use std::vec::Vec;

pub mod joystick;
pub mod keymap;
pub mod mouse;

mod parsing;

use joystick::Joystick_Button;
use mouse::Mouse_Button;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Input_Action {
    Key(keymap::Key),
    Joystick(Joystick_Button),
    Mouse(Mouse_Button),
}

/// Struct containing the mappings between input and user-defined actions and axes_mappings.
/// e.g. "Key::Q => action_quit".
pub struct Input_Bindings {
    /// { input_action => [action_name] }
    action_bindings: HashMap<Input_Action, Vec<String_Id>>,
    /// { input_axis_mapping => [axis_name] }
    axis_bindings: HashMap<Virtual_Axis_Mapping, Vec<String_Id>>,
}

impl Input_Bindings {
    pub fn create_from_config(
        action_bindings_file: &Path,
        axis_bindings_file: &Path,
    ) -> Result<Input_Bindings, String> {
        Ok(Input_Bindings {
            action_bindings: parsing::parse_action_bindings_file(action_bindings_file)?,
            axis_bindings: parsing::parse_axis_bindings_file(axis_bindings_file)?,
        })
    }

    pub fn get_key_action(&self, code: keymap::Key) -> Option<&Vec<String_Id>> {
        self.action_bindings.get(&Input_Action::Key(code))
    }

    pub fn get_joystick_action(&self, joystick_id: u32, button: u32) -> Option<&Vec<String_Id>> {
        // @Incomplete: retrieve the correct joystick from a joystick manager or something.
        let joystick = joystick::Joystick {
            id: 0,
            joy_type: joystick::Joystick_Type::XBox360,
        };
        self.action_bindings
            .get(&Input_Action::Joystick(joystick::get_joy_btn_from_id(
                joystick, button,
            )?))
    }

    pub fn get_mouse_action(&self, button: mouse::Button) -> Option<&Vec<String_Id>> {
        let button = mouse::get_mouse_btn(button)?;
        self.action_bindings.get(&Input_Action::Mouse(button))
    }
}
