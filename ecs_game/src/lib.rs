extern crate ecs_engine;

use ecs_engine::core::{self, app, app_config};
use ecs_engine::gfx::{self, window};

use std::env;

pub struct Game_State {
    pub window: window::Window_Handle,
    pub engine_state: Engine_State,
}

#[no_mangle]
pub extern "C" fn game_init() -> *mut Game_State {
    if let Ok(state) = internal_game_init() {
        &mut state
    } else {
        std::ptr::null
    }
}

fn internal_game_init() -> Result<Game_State, Box<dyn std::error::Error>> {
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

    app::start_game_loop(&mut engine_state, &mut window)?;

    Ok(Game_State {
        window,
        engine_state,
    })
}
