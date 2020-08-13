#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate inle_math;

mod drawing;
pub mod ui_context;
pub mod widgets;

pub use drawing::draw_all_ui;
pub use ui_context::*;
pub use widgets::*;
