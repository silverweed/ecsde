#![allow(non_camel_case_types)]

use std::env;

pub(crate) mod alloc;
pub(crate) mod audio;
pub(crate) mod core;
pub(crate) mod ecs;
pub(crate) mod game;
pub(crate) mod gfx;
pub(crate) mod resources;

fn main() -> core::common::Maybe_Error {
    let cfg = core::app::Config::new(env::args());
    let mut app = core::app::App::new(&cfg);

    app.init()?;
    app.run()
}
