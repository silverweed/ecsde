use crate::core::common::stringid::String_Id;
use crate::input::actions::Action_List;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::vec::Vec;

pub(super) type Action_Callback = Box<dyn Fn(&mut Action_List)>;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub(super) enum Action_Kind {
    Pressed,
    Released,
}

/// Struct containing the mappings between user-defined actions and callbacks.
/// e.g. "action_quit => (quit the game)"
pub(super) struct Action_Callbacks {
    mappings: HashMap<(String_Id, Action_Kind), Vec<Action_Callback>>,
}

impl Action_Callbacks {
    pub fn new() -> Action_Callbacks {
        Action_Callbacks {
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
