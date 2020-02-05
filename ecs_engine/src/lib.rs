#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

extern crate byteorder;
extern crate cgmath;
extern crate crossbeam;
extern crate num_cpus;
extern crate num_enum;

#[cfg(debug_assertions)]
extern crate notify;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
extern crate float_cmp;

#[macro_use]
extern crate bitflags;

#[cfg(features = "use-sfml")]
extern crate sfml;

#[macro_use]
pub mod prelude;

pub mod alloc;
pub mod audio;
pub mod cfg;
pub mod collisions;
pub mod core;
pub mod ecs;
pub mod fs;
pub mod gfx;
pub mod input;
pub mod replay;
pub mod resources;

// Note: if the following line is uncommented, dependant crates won't import the module
// correctly. Investigate on this.
//#[cfg(any(test, debug_assertions))]
pub mod test_common;

#[cfg(debug_assertions)]
pub mod debug;
