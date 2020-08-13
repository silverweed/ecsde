#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate inle_common;

#[macro_use]
extern crate inle_diagnostics;

pub mod axes;
pub mod bindings;
pub mod core_actions;
pub mod events;
pub mod input_state;
pub mod joystick;
pub mod joystick_state;
pub mod keyboard;
pub mod mouse;
pub mod serialize;

use inle_core::env::Env_Info;

// @Incomplete: allow selecting file path
fn create_bindings(env: &Env_Info) -> bindings::Input_Bindings {
    let mut action_bindings_path = std::path::PathBuf::new();
    action_bindings_path.push(&env.cfg_root);
    action_bindings_path.push("input");
    action_bindings_path.set_extension("actions");
    let mut axis_bindings_path = std::path::PathBuf::new();
    axis_bindings_path.push(&env.cfg_root);
    axis_bindings_path.push("input");
    axis_bindings_path.set_extension("axes");
    bindings::Input_Bindings::create_from_config(&action_bindings_path, &axis_bindings_path)
        .unwrap()
}
