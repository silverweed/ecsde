use super::joystick_mgr::Joystick_State;
use crate::common::stringid::String_Id;
use std::collections::HashMap;
use std::path::Path;
use std::vec::Vec;

pub mod joystick;
pub mod keyboard;
pub mod mouse;

mod parsing;

use joystick::Joystick_Button;
use mouse::Mouse_Button;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Input_Action {
    Key(keyboard::Key),
    Joystick(Joystick_Button),
    Mouse(Mouse_Button),
    /// positive is up, negative is down.
    Mouse_Wheel {
        positive: bool,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Axis_Emulation_Type {
    Min,
    Max,
}

impl Axis_Emulation_Type {
    #[inline]
    pub fn assoc_value(self) -> f32 {
        match self {
            Axis_Emulation_Type::Min => -1.0,
            Axis_Emulation_Type::Max => 1.0,
        }
    }
}

pub struct Axis_Bindings {
    pub axes_names: Vec<String_Id>,
    pub real: [Vec<String_Id>; joystick::Joystick_Axis::_Count as usize],
    pub emulated: HashMap<Input_Action, Vec<(String_Id, Axis_Emulation_Type)>>,
}

/// Struct containing the mappings between input and user-defined actions and axes_mappings.
/// e.g. "Key::Q => action_quit".
pub struct Input_Bindings {
    /// { input_action => [action_name] }
    pub action_bindings: HashMap<Input_Action, Vec<String_Id>>,
    pub axis_bindings: Axis_Bindings,
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

    pub fn get_virtual_axes_from_real_axis(
        &self,
        real_axis: joystick::Joystick_Axis,
    ) -> &[String_Id] {
        &self.axis_bindings.real[real_axis as usize]
    }

    /// Makes an inverse search in the bindings, returning all kinds of actions that yield the given `action_name`.
    #[cfg(debug_assertions)]
    pub fn get_all_actions_triggering(&self, action_name: String_Id) -> Vec<Input_Action> {
        self.action_bindings
            .iter()
            .filter_map(|(action, names)| {
                if names.contains(&action_name) {
                    Some(*action)
                } else {
                    None
                }
            })
            .collect()
    }

    pub(super) fn get_key_actions(&self, code: keyboard::Key) -> Option<&Vec<String_Id>> {
        self.action_bindings.get(&Input_Action::Key(code))
    }

    pub(super) fn get_joystick_actions(
        &self,
        joystick_id: u32,
        button: u32,
        joy_state: &Joystick_State,
    ) -> Option<&[String_Id]> {
        let joystick = &joy_state.joysticks[joystick_id as usize].unwrap_or_else(|| {
            fatal!(
                "Tried to get action for joystick {}, but it is not registered!",
                joystick_id
            )
        });
	let joystick = joystick::get_joy_btn_from_id(*joystick, button)?;
	let input_action = Input_Action::Joystick(joystick);
        self.action_bindings.get(&input_action).map(Vec::as_slice)
    }

    pub(super) fn get_mouse_actions(&self, button: mouse::Button) -> Option<&Vec<String_Id>> {
        let button = mouse::get_mouse_btn(button)?;
        self.action_bindings.get(&Input_Action::Mouse(button))
    }

    pub(super) fn get_mouse_wheel_actions(&self, positive: bool) -> Option<&Vec<String_Id>> {
        self.action_bindings
            .get(&Input_Action::Mouse_Wheel { positive })
    }

    pub(super) fn get_key_emulated_axes(
        &self,
        code: keyboard::Key,
    ) -> Option<&Vec<(String_Id, Axis_Emulation_Type)>> {
        self.axis_bindings.emulated.get(&Input_Action::Key(code))
    }

    pub(super) fn get_joystick_emulated_axes(
        &self,
        joystick_id: u32,
        button: u32,
        joy_state: &Joystick_State,
    ) -> Option<&Vec<(String_Id, Axis_Emulation_Type)>> {
        let joystick = &joy_state.joysticks[joystick_id as usize].unwrap_or_else(|| {
            panic!(
                "[ ERROR ] Tried to get emulated axes for joystick {}, but it is not registered!",
                joystick_id
            )
        });
        self.axis_bindings
            .emulated
            .get(&Input_Action::Joystick(joystick::get_joy_btn_from_id(
                *joystick, button,
            )?))
    }

    pub(super) fn get_mouse_emulated_axes(
        &self,
        button: mouse::Button,
    ) -> Option<&Vec<(String_Id, Axis_Emulation_Type)>> {
        let button = mouse::get_mouse_btn(button)?;
        self.axis_bindings
            .emulated
            .get(&Input_Action::Mouse(button))
    }

    pub(super) fn get_mouse_wheel_emulated_axes(
        &self,
        positive: bool,
    ) -> Option<&Vec<(String_Id, Axis_Emulation_Type)>> {
        self.axis_bindings
            .emulated
            .get(&Input_Action::Mouse_Wheel { positive })
    }
}