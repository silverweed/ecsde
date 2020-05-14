#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate bitflags;

#[cfg(debug_assertions)]
#[macro_use]
extern crate lazy_static;

#[macro_use]
pub mod prelude;

// Note: if the following line is uncommented, dependant crates won't import the module
// correctly. Investigate on this.
//#[cfg(any(test, debug_assertions))]
#[macro_use]
pub mod test_common;

pub mod alloc;
pub mod audio;
pub mod cfg;
pub mod collisions;
pub mod common;
pub mod core;
pub mod ecs;
pub mod events;
pub mod fs;
pub mod ui;
pub mod gfx;
pub mod input;
pub mod replay;
pub mod resources;

#[cfg(debug_assertions)]
pub mod debug;
