#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[cfg(test)]
#[macro_use]
extern crate inle_test;

#[macro_use]
extern crate inle_diagnostics;

#[macro_use]
pub mod prelude;

pub mod angle;
pub mod math;
pub mod matrix;
pub mod rect;
pub mod shapes;
pub mod transform;
pub mod vector;

pub use prelude::*;
