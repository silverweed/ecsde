#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

extern crate byteorder;
extern crate cgmath;
extern crate ears;
extern crate notify;
extern crate num_enum;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
extern crate float_cmp;

#[macro_use]
extern crate bitflags;

#[cfg(features = "use-sfml")]
extern crate sfml;

pub mod alloc;
pub mod audio;
pub mod cfg;
pub mod core;
pub mod fs;
pub mod gfx;
pub mod input;
pub mod replay;
pub mod resources;

#[cfg(debug_assertions)]
pub mod test_common;

#[cfg(debug_assertions)]
pub mod debug;
