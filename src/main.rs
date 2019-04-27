#![allow(non_camel_case_types)]

extern crate anymap;
extern crate cgmath;
extern crate ears;
extern crate sdl2;
extern crate stb_image;

use std::env;

pub(crate) mod alloc;
pub(crate) mod audio;
pub(crate) mod core;
pub(crate) mod ecs;
pub(crate) mod game;
pub(crate) mod gfx;
pub(crate) mod resources;
#[cfg(test)]
pub(crate) mod test_common;

fn main() -> core::common::Maybe_Error {
    let cfg = core::app::Config::new(env::args());
    let mut app = core::app::App::new(&cfg);

    app.init()?;
    app.run()
}
