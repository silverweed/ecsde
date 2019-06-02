#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

extern crate anymap;
extern crate cgmath;
extern crate ears;
extern crate notify;
extern crate rand;
#[cfg(features = "use-sdl")]
extern crate sdl2;
#[cfg(features = "use-sfml")]
extern crate sfml;

use std::env;

pub(crate) mod alloc;
pub(crate) mod audio;
pub(crate) mod cfg;
pub(crate) mod core;
pub(crate) mod ecs;
pub(crate) mod fs;
pub(crate) mod game;
pub(crate) mod gfx;
pub(crate) mod resources;
#[cfg(test)]
pub(crate) mod test_common;

fn main() -> core::common::Maybe_Error {
    let cfg = core::app::App_Config::new(env::args());

    let sound_loader = audio::sound_loader::Sound_Loader {};
    let mut app = core::app::App::new(&sound_loader);

    app.init()?;
    app.run()
}
