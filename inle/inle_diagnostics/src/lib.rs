#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[cfg(feature = "tracer")]
#[macro_use]
extern crate lazy_static;

#[macro_use]
pub mod prelude;

#[cfg(feature = "tracer")]
pub mod tracer;

pub mod log;

pub use prelude::*;
