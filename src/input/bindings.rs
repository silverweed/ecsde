use crate::core::common::stringid::String_Id;
use super::joystick_mgr::Joystick_Manager;
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(super) enum Axis_Emulation_Type {
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

pub(super) struct Axis_Bindings {
    pub(self) axes_names: Vec<String_Id>,
    pub(self) real: [Vec<String_Id>; joystick::Joystick_Axis::_Count as usize],
    pub(self) emulated: HashMap<Input_Action, Vec<(String_Id, Axis_Emulation_Type)>>,
}

/// Struct containing the mappings between input and user-defined actions and axes_mappings.
/// e.g. "Key::Q => action_quit".
pub struct Input_Bindings {
    /// { input_action => [action_name] }
    action_bindings: HashMap<Input_Action, Vec<String_Id>>,
    axis_bindings: Axis_Bindings,
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

    pub fn get_all_virtual_axes_names(&self) -> &[String_Id] {
        &self.axis_bindings.axes_names
    }

    pub fn get_virtual_axes_from_real_axis(
        &self,
        real_axis: joystick::Joystick_Axis,
    ) -> &[String_Id] {
        &self.axis_bindings.real[real_axis as usize]
    }

    pub(super) fn get_key_actions(&self, code: keymap::Key) -> Option<&Vec<String_Id>> {
        self.action_bindings.get(&Input_Action::Key(code))
    }

    pub(super) fn get_joystick_actions(
        &self,
        joystick_id: u32,
        button: u32,
		joy_mgr: &Joystick_Manager,
    ) -> Option<&Vec<String_Id>> {
        let joystick = joy_mgr.get_joystick(joystick_id).unwrap_or_else(||
			panic!("[ ERROR ] Tried to get action for joystick {}, but it is not registered!", joystick_id)
		);
        self.action_bindings
            .get(&Input_Action::Joystick(joystick::get_joy_btn_from_id(
                *joystick, button,
            )?))
    }

    pub(super) fn get_mouse_actions(&self, button: mouse::Button) -> Option<&Vec<String_Id>> {
        let button = mouse::get_mouse_btn(button)?;
        self.action_bindings.get(&Input_Action::Mouse(button))
    }

    pub(super) fn get_key_emulated_axes(
        &self,
        code: keymap::Key,
    ) -> Option<&Vec<(String_Id, Axis_Emulation_Type)>> {
        self.axis_bindings.emulated.get(&Input_Action::Key(code))
    }

    pub(super) fn get_joystick_emulated_axes(
        &self,
        joystick_id: u32,
        button: u32,
		joy_mgr: &Joystick_Manager,
    ) -> Option<&Vec<(String_Id, Axis_Emulation_Type)>> {
        let joystick = joy_mgr.get_joystick(joystick_id).unwrap_or_else(||
			panic!("[ ERROR ] Tried to get emulated axes for joystick {}, but it is not registered!", joystick_id)
		);
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
}
