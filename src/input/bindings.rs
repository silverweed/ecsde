use crate::core::common::stringid::String_Id;
use crate::input::actions::Action_List;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::Path;
use std::vec::Vec;

mod joystick;
mod keymap;
mod mouse;
mod parsing;

use joystick::Joystick_Button;
use mouse::Mouse_Button;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Input_Action {
    Key(keymap::Key),
    Joystick(Joystick_Button),
    Mouse(Mouse_Button),
}

pub struct Input_Bindings {
    /// { input_action => [action_name] }
    action_bindings: HashMap<Input_Action, Vec<String_Id>>,
}

impl Input_Bindings {
    pub fn create_from_config(bindings_file_path: &Path) -> Result<Input_Bindings, String> {
        Ok(Input_Bindings {
            action_bindings: parsing::parse_bindings_file(bindings_file_path)?,
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

    #[cfg(feature = "use-sfml")]
    pub fn get_mouse_action(&self, button: sfml::window::mouse::Button) -> Option<&Vec<String_Id>> {
        let button = Mouse_Button::try_from(button).ok()?;
        self.action_bindings.get(&Input_Action::Mouse(button))
    }
}

pub(super) type Action_Callback = Box<dyn Fn(&mut Action_List)>;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub(super) enum Action_Kind {
    Pressed,
    Released,
}

pub(super) struct Action_Mappings {
    mappings: HashMap<(String_Id, Action_Kind), Vec<Action_Callback>>,
}

impl Action_Mappings {
    pub fn new() -> Action_Mappings {
        Action_Mappings {
            mappings: HashMap::new(),
        }
    }

    pub fn register_mapping(
        &mut self,
        action_name: String_Id,
        action_kind: Action_Kind,
        callback: impl Fn(&mut Action_List) + 'static,
    ) {
        match self.mappings.entry((action_name, action_kind)) {
            Entry::Occupied(mut o) => o.get_mut().push(Box::new(callback)),
            Entry::Vacant(v) => {
                v.insert(vec![Box::new(callback)]);
            }
        }
    }

    pub fn get_callbacks_for_action(
        &self,
        action_name: String_Id,
        action_kind: Action_Kind,
    ) -> Option<&Vec<Action_Callback>> {
        self.mappings.get(&(action_name, action_kind))
    }
}
