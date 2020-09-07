#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate inle_diagnostics;

#[macro_use]
extern crate inle_math;

#[macro_use]
pub mod prelude;

mod comp_mgr;

pub mod components;
pub mod ecs_world;
pub mod entity_stream;
pub mod system;

pub use prelude::*;
