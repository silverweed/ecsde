#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

extern crate anymap;
extern crate cgmath;
extern crate ears;
extern crate notify;
extern crate sdl2;

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
    let cfg = core::app::Config::new(env::args());

    let sdl = sdl2::init().unwrap();
    let event_pump = sdl.event_pump().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let mut window =
        gfx::window::create_render_window(&video_subsystem, cfg.target_win_size, &cfg.title);

    let resource_loaders = core::app::Resource_Loaders {
        texture_creator: window.texture_creator(),
        ttf_context: sdl2::ttf::init().unwrap(),
        sound_loader: audio::sound_loader::Sound_Loader {},
    };

    let mut app = core::app::App::new(event_pump, &resource_loaders);

    app.init()?;
    app.run(&mut window)
}
