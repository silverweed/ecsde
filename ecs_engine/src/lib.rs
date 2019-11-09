#![warn(clippy::all)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

extern crate anymap;
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
pub mod states;

#[cfg(test)]
pub(crate) mod test_common;

#[cfg(debug_assertions)]
pub(crate) mod debug;
