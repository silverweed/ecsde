#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
pub mod prelude;

pub mod angle;
pub mod transform;
pub mod matrix;
pub mod math;
pub mod vector;
pub mod shapes;

pub use prelude::*;
