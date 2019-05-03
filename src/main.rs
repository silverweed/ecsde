#![allow(non_camel_case_types)]

extern crate anymap;
extern crate cgmath;
extern crate ears;
extern crate sdl2;

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

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let mut canvas =
        gfx::window::create_render_canvas(&video_subsystem, cfg.target_win_size, &cfg.title);

    let resource_loaders = core::app::Resource_Loaders {
        texture_creator: canvas.texture_creator(),
        ttf_context: sdl2::ttf::init().unwrap(),
        sound_loader: audio::sound_loader::Sound_Loader {},
    };

    let mut app = core::app::App::new(&cfg, &sdl, &mut canvas, &resource_loaders);

    app.init()?;
    app.run()
}
