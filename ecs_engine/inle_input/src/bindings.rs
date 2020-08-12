use super::joystick::{self, Joystick_Button};
use super::joystick_state::Joystick_State;
use super::keyboard::Key;
use super::mouse::{self, Mouse_Button};
use crate::common::stringid::String_Id;
use std::collections::HashMap;
use std::path::Path;
use std::vec::Vec;

mod parsing;

use self::modifiers::*;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Input_Action_Simple {
    Key(Key),
    Joystick(Joystick_Button),
    Mouse(Mouse_Button),
    Mouse_Wheel { up: bool },
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Input_Action {
    pub action: Input_Action_Simple,
    pub modifiers: Input_Action_Modifiers,
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

    pub(super) fn get_key_actions(
        &self,
        code: Key,
        modifiers: Input_Action_Modifiers,
    ) -> Option<&Vec<String_Id>> {
        self.action_bindings.get(&Input_Action::new_with_modifiers(
            Input_Action_Simple::Key(code),
            modifiers,
        ))
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
        // @Incomplete: do we want to support modifiers on joysticks?
        let input_action = Input_Action::new(Input_Action_Simple::Joystick(joystick));
        self.action_bindings.get(&input_action).map(Vec::as_slice)
    }

    pub(super) fn get_mouse_actions(
        &self,
        button: mouse::Mouse_Button,
        modifiers: Input_Action_Modifiers,
    ) -> Option<&Vec<String_Id>> {
        self.action_bindings.get(&Input_Action::new_with_modifiers(
            Input_Action_Simple::Mouse(button),
            modifiers,
        ))
    }

    pub(super) fn get_mouse_wheel_actions(
        &self,
        up: bool,
        modifiers: Input_Action_Modifiers,
    ) -> Option<&Vec<String_Id>> {
        self.action_bindings.get(&Input_Action::new_with_modifiers(
            Input_Action_Simple::Mouse_Wheel { up },
            modifiers,
        ))
    }

    pub(super) fn get_key_emulated_axes(
        &self,
        code: Key,
    ) -> Option<&Vec<(String_Id, Axis_Emulation_Type)>> {
        self.axis_bindings
            .emulated
            .get(&Input_Action::new(Input_Action_Simple::Key(code)))
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
            .get(&Input_Action::new(Input_Action_Simple::Joystick(
                joystick::get_joy_btn_from_id(*joystick, button)?,
            )))
    }

    pub(super) fn get_mouse_emulated_axes(
        &self,
        button: mouse::Mouse_Button,
    ) -> Option<&Vec<(String_Id, Axis_Emulation_Type)>> {
        self.axis_bindings
            .emulated
            .get(&Input_Action::new(Input_Action_Simple::Mouse(button)))
    }

    pub(super) fn get_mouse_wheel_emulated_axes(
        &self,
        up: bool,
    ) -> Option<&Vec<(String_Id, Axis_Emulation_Type)>> {
        self.axis_bindings
            .emulated
            .get(&Input_Action::new(Input_Action_Simple::Mouse_Wheel { up }))
    }
}

pub type Input_Action_Modifiers = u8;

// @WaitForStable: make this const
pub fn input_action_modifier_from_key(key: Key) -> Input_Action_Modifiers {
    match key {
        Key::LControl => MOD_LCTRL,
        Key::RControl => MOD_RCTRL,
        Key::LShift => MOD_LSHIFT,
        Key::RShift => MOD_RSHIFT,
        Key::LAlt => MOD_LALT,
        Key::RAlt => MOD_RALT,
        Key::LSystem => MOD_LSUPER,
        Key::RSystem => MOD_RSUPER,
        _ => 0,
    }
}

pub mod modifiers {
    use super::Input_Action_Modifiers;

    #[allow(clippy::identity_op)]
    pub const MOD_LCTRL: Input_Action_Modifiers = 1 << 0;
    pub const MOD_RCTRL: Input_Action_Modifiers = 1 << 1;
    pub const MOD_CTRL: Input_Action_Modifiers = MOD_LCTRL | MOD_RCTRL;
    pub const MOD_LSHIFT: Input_Action_Modifiers = 1 << 2;
    pub const MOD_RSHIFT: Input_Action_Modifiers = 1 << 3;
    pub const MOD_SHIFT: Input_Action_Modifiers = MOD_LSHIFT | MOD_RSHIFT;
    pub const MOD_LALT: Input_Action_Modifiers = 1 << 4;
    pub const MOD_RALT: Input_Action_Modifiers = 1 << 5;
    pub const MOD_ALT: Input_Action_Modifiers = MOD_LALT | MOD_RALT;
    pub const MOD_LSUPER: Input_Action_Modifiers = 1 << 6;
    pub const MOD_RSUPER: Input_Action_Modifiers = 1 << 7;
    pub const MOD_SUPER: Input_Action_Modifiers = MOD_LSUPER | MOD_RSUPER;
}

impl Input_Action {
    pub fn new(action: Input_Action_Simple) -> Self {
        Self {
            action,
            modifiers: 0,
        }
    }

    pub fn new_with_modifiers(
        action: Input_Action_Simple,
        modifiers: Input_Action_Modifiers,
    ) -> Self {
        Self { action, modifiers }
    }

    #[inline]
    pub fn has_modifiers(&self) -> bool {
        self.modifiers != 0
    }

    #[inline]
    pub fn has_ctrl(&self) -> bool {
        (self.modifiers & MOD_CTRL) != 0
    }

    #[inline]
    pub fn has_lctrl(&self) -> bool {
        (self.modifiers & MOD_LCTRL) != 0
    }

    #[inline]
    pub fn has_rctrl(&self) -> bool {
        (self.modifiers & MOD_RCTRL) != 0
    }

    #[inline]
    pub fn has_shift(&self) -> bool {
        (self.modifiers & MOD_SHIFT) != 0
    }

    #[inline]
    pub fn has_lshift(&self) -> bool {
        (self.modifiers & MOD_LSHIFT) != 0
    }

    #[inline]
    pub fn has_rshift(&self) -> bool {
        (self.modifiers & MOD_RSHIFT) != 0
    }

    #[inline]
    pub fn has_alt(&self) -> bool {
        (self.modifiers & MOD_ALT) != 0
    }

    #[inline]
    pub fn has_lalt(&self) -> bool {
        (self.modifiers & MOD_LALT) != 0
    }

    #[inline]
    pub fn has_altgr(&self) -> bool {
        (self.modifiers & MOD_RALT) != 0
    }

    #[inline]
    pub fn has_super(&self) -> bool {
        (self.modifiers & MOD_SUPER) != 0
    }

    #[inline]
    pub fn has_lsuper(&self) -> bool {
        (self.modifiers & MOD_LSUPER) != 0
    }

    #[inline]
    pub fn has_rsuper(&self) -> bool {
        (self.modifiers & MOD_RSUPER) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifier_from_key() {
        assert_eq!(input_action_modifier_from_key(Key::LAlt), MOD_LALT);
        assert_eq!(input_action_modifier_from_key(Key::RAlt), MOD_RALT);
        assert_eq!(input_action_modifier_from_key(Key::LControl), MOD_LCTRL);
        assert_eq!(input_action_modifier_from_key(Key::RControl), MOD_RCTRL);
        assert_eq!(input_action_modifier_from_key(Key::LSystem), MOD_LSUPER);
        assert_eq!(input_action_modifier_from_key(Key::RSystem), MOD_RSUPER);
        assert_eq!(input_action_modifier_from_key(Key::LShift), MOD_LSHIFT);
        assert_eq!(input_action_modifier_from_key(Key::RShift), MOD_RSHIFT);
        assert_eq!(input_action_modifier_from_key(Key::Space), 0);
        assert_eq!(input_action_modifier_from_key(Key::H), 0);
    }
}
