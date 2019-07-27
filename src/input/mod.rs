pub mod actions;
pub mod bindings;
pub mod input_system;

use crate::core::common::direction::Direction;
use crate::core::common::vector::Vec2f;
use crate::core::env::Env_Info;
use cgmath::InnerSpace;

pub fn get_movement_from_input(actions: &actions::Action_List) -> Vec2f {
    let mut movement = Vec2f::new(0.0, 0.0);
    if actions.has_action(&actions::Action::Move(Direction::Left)) {
        movement.x -= 1.0;
    }
    if actions.has_action(&actions::Action::Move(Direction::Right)) {
        movement.x += 1.0;
    }
    if actions.has_action(&actions::Action::Move(Direction::Up)) {
        movement.y -= 1.0;
    }
    if actions.has_action(&actions::Action::Move(Direction::Down)) {
        movement.y += 1.0;
    }
    movement
}

pub fn get_normalized_movement_from_input(actions: &actions::Action_List) -> Vec2f {
    let m = get_movement_from_input(actions);
    if m.magnitude2() == 0.0 {
        m
    } else {
        m.normalize()
    }
}

// @Incomplete: allow selecting file path
fn create_bindings(env: &Env_Info) -> bindings::Input_Bindings {
    let mut keybinding_path = std::path::PathBuf::new();
    keybinding_path.push(env.get_cfg_root());
    keybinding_path.push("input");
    keybinding_path.set_extension("keys");
    bindings::Input_Bindings::create_from_config(&keybinding_path).unwrap()
}
