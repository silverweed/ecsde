#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate inle_diagnostics;

#[macro_use]
pub mod prelude;

pub mod bitset;
pub mod colors;
pub mod fixed_string;
pub mod paint_props;
pub mod stringid;
pub mod thread_safe_ptr;
pub mod units;
pub mod variant;

pub type Maybe_Error = Result<(), Box<dyn std::error::Error>>;
pub const WORD_SIZE: usize = std::mem::size_of::<usize>();

pub use prelude::*;
