#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

extern crate ecs_engine;

mod game_loop;

use ecs_engine::cfg::Cfg_Var;
use ecs_engine::core::{app, app_config};
use ecs_engine::debug;
use ecs_engine::gfx::{self, window};
use std::env;
use std::time::Duration;

#[repr(C)]
pub struct Game_State<'a> {
    pub window: window::Window_Handle,
    pub engine_state: app::Engine_State<'a>,

    //#[cfg(debug_assertions)]
    //pub fps_debug: debug::fps::Fps_Console_Printer,

    //pub execution_time: Duration,
    //input_provider = create_input_provider(&mut engine_state.replay_data);
    //is_replaying = !input_provider.is_realtime_player_input();
    //// Cfg vars
    //let update_time = Cfg_Var::<i32>::new("engine/gameplay/gameplay_update_tick_ms");
    //let smooth_by_extrapolating_velocity =
    //Cfg_Var::<bool>::new("engine/rendering/smooth_by_extrapolating_velocity");
    //#[cfg(debug_assertions)]
    //let extra_frame_sleep_ms = Cfg_Var::<i32>::new("engine/debug/extra_frame_sleep_ms");
    //#[cfg(debug_assertions)]
    //let record_replay = Cfg_Var::<bool>::new("engine/debug/replay/record");

    //#[cfg(debug_assertions)]
    //let sid_joysticks = String_Id::from("joysticks");
    //#[cfg(debug_assertions)]
    //let sid_msg = String_Id::from("msg");
    //#[cfg(debug_assertions)]
    //let sid_time = String_Id::from("time");
    //#[cfg(debug_assertions)]
    //let sid_fps = String_Id::from("fps");
}

// Note: the lifetime is actually ignored
#[no_mangle]
pub extern "C" fn game_init<'a>() -> *mut Game_State<'a> {
    eprintln!("[ INFO ] Initializing game...");
    if let Ok(game_state) = internal_game_init() {
        Box::into_raw(game_state)
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn game_update(game_state: *mut Game_State) -> bool {
    if game_state.is_null() {
        panic!("[ FATAL ] game_update: game state is null!");
    }

    let game_state = &mut *game_state;
    let engine_state = &mut game_state.engine_state;
    if engine_state.should_close {
        return false;
    }

    engine_state.time.update();

    println!(
        "real time: {:?}; game time: {:?}",
        engine_state.time.get_real_time(),
        engine_state.time.get_game_time()
    );

    std::thread::sleep(std::time::Duration::from_millis(16));

    true
}

#[no_mangle]
pub unsafe extern "C" fn game_shutdown(game_state: *mut Game_State) {
    if game_state.is_null() {
        panic!("[ FATAL ] game_shutdown: game state is null!");
    }

    std::ptr::drop_in_place(game_state);

    eprintln!("[ OK ] Game was shut down.");
}

fn internal_game_init<'a>() -> Result<Box<Game_State<'a>>, Box<dyn std::error::Error>> {
    let cfg = app_config::App_Config::new(env::args());

    let window = gfx::window::create_render_window(&(), cfg.target_win_size, &cfg.title);
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

    Ok(Box::new(Game_State {
        window,
        engine_state,
    }))
}
