pub mod actions;
pub mod axes;
pub mod bindings;
pub mod callbacks;
pub mod input_system;
pub mod provider;

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
    let mut action_bindings_path = std::path::PathBuf::new();
    action_bindings_path.push(env.get_cfg_root());
    action_bindings_path.push("input");
    action_bindings_path.set_extension("actions");
    let mut axis_bindings_path = std::path::PathBuf::new();
    axis_bindings_path.push(env.get_cfg_root());
    axis_bindings_path.push("input");
    axis_bindings_path.set_extension("axes");
    bindings::Input_Bindings::create_from_config(&action_bindings_path, &axis_bindings_path)
        .unwrap()
}
