use crate::core::common::direction::{Direction, Direction_Flags};
use std::vec::Vec;

#[derive(PartialEq, Hash, Copy, Clone, Debug)]
pub enum Action {
    Quit,
    Resize(u32, u32),
    Move(Direction),
    // Note: the zoom factor is an integer rather than a float as it can be hashed.
    // This integer must be divided by 100 to obtain the actual scaling factor.
    Zoom(i32),
    Change_Speed(i32),
    Pause_Toggle,
    Step_Simulation,
    Print_Entity_Manager_Debug_Info,
}

#[derive(Default, Clone, Debug)]
pub struct Action_List {
    pub(super) directions: Direction_Flags,
    pub(super) actions: Vec<Action>,
}

impl Action_List {
    pub fn has_action(&self, action: &Action) -> bool {
        match action {
            Action::Move(Direction::Up) => self.directions.contains(Direction_Flags::UP),
            Action::Move(Direction::Left) => self.directions.contains(Direction_Flags::LEFT),
            Action::Move(Direction::Down) => self.directions.contains(Direction_Flags::DOWN),
            Action::Move(Direction::Right) => self.directions.contains(Direction_Flags::RIGHT),
            _ => self.actions.contains(&action),
        }
    }

    pub fn get_directions(&self) -> Direction_Flags {
        self.directions
    }
}

impl std::ops::Deref for Action_List {
    type Target = <Vec<Action> as std::ops::Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.actions.deref()
    }
}
