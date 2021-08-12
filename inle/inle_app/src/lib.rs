#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate inle_common;

#[macro_use]
extern crate inle_diagnostics;

#[macro_use]
extern crate inle_ecs;

#[macro_use]
extern crate inle_math;

pub mod animation_system;
pub mod app;
pub mod app_config;
pub mod render_system;
pub mod systems;
