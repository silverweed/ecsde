#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

pub extern crate float_cmp;

#[macro_use]
pub mod prelude;

pub mod approx_eq_testable;
pub mod env;
pub mod test_common;

pub use prelude::*;
