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

#[macro_use]
extern crate inle_resources;

//pub mod components;
pub mod light;
pub mod material;
pub mod particles;
pub mod render;
pub mod render_window;
pub mod res;
pub mod sprites;
pub mod vbuf_holder;
