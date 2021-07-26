#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[cfg(test)]
#[macro_use]
extern crate inle_test;

#[macro_use]
extern crate inle_common;

#[macro_use]
extern crate inle_diagnostics;

#[macro_use]
extern crate inle_math;

pub_in_debug! {mod backend_common;} // this needs to be exported for the buf_alloc_debug
pub mod render;
pub mod render_window;
