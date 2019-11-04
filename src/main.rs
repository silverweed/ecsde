#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

extern crate anymap;
extern crate byteorder;
extern crate cgmath;
extern crate ears;
extern crate notify;
extern crate num_enum;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
extern crate float_cmp;

#[macro_use]
extern crate bitflags;

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
pub(crate) mod input;
pub(crate) mod replay;
pub(crate) mod resources;
pub(crate) mod states;

#[cfg(test)]
pub(crate) mod test_common;

#[cfg(debug_assertions)]
pub(crate) mod debug;

use self::core::{app, app_config};

fn main() -> core::common::Maybe_Error {
    let cfg = app_config::App_Config::new(env::args());

    let mut window = gfx::window::create_render_window(&(), cfg.target_win_size, &cfg.title);
    let mut engine_state = app::create_engine_state(cfg);

    println!(
        "Working dir = {:?}\nExe = {:?}",
        engine_state.env.get_cwd(),
        engine_state.env.get_exe()
    );

    app::init_engine_systems(&mut engine_state)?;
    app::start_config_watch(&engine_state.env, &mut engine_state.config)?;

    #[cfg(debug_assertions)]
    {
        app::init_engine_debug(&mut engine_state)?;
        app::start_recording(
            &engine_state.replay_data,
            &mut engine_state.debug_systems.replay_recording_system,
        )?;
    }

    app::start_game_loop(&mut engine_state, &mut window)
}
