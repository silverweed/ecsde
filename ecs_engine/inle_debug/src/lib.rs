#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate inle_common;

#[cfg(features = "full")]
pub mod calipers;
#[cfg(features = "full")]
pub mod console;
#[cfg(features = "full")]
pub mod debug_ui;
#[cfg(features = "full")]
pub mod element;
#[cfg(features = "full")]
pub mod fadeout_overlay;
#[cfg(features = "full")]
pub mod fps;
#[cfg(features = "full")]
pub mod frame_scroller;
#[cfg(features = "full")]
pub mod graph;
#[cfg(features = "full")]
pub mod log;
#[cfg(features = "full")]
pub mod overlay;
#[cfg(features = "full")]
pub mod painter;
