#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate inle_common;

#[macro_use]
extern crate inle_diagnostics;

#[macro_use]
extern crate inle_math;

#[cfg(debug_assertions)]
pub mod backend_specific_debugs;

pub mod calipers;
pub mod console;
pub mod debug_ui;
pub mod element;
pub mod fps;
pub mod frame_scroller;
pub mod graph;
pub mod log;
pub mod log_window;
pub mod overlay;
pub mod painter;
