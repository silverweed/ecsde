extern crate ecs_engine;

use ecs_engine::core::{self, app, app_config};
use ecs_engine::gfx;

use std::env;

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
