#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

extern crate ecs_engine;

mod game_loop;

use ecs_engine::core::{app, app_config};
use ecs_engine::gfx::{self, window};
use std::env;
use std::time::Instant;

#[repr(C)]
pub struct Game_State {
    pub creat_time: Instant,
    pub iter: u64,
    pub prelude: String,
    //pub window: window::Window_Handle,
    //pub engine_state: app::Engine_State<'a>,
}

#[no_mangle]
pub extern "C" fn game_init() -> *mut Game_State {
    let game_state = Box::new(Game_State {
        creat_time: Instant::now(),
        prelude: String::from("Game says:"),
        iter: 0,
    });
    eprintln!("[ INFO ] Initializing game...");
    Box::into_raw(game_state)
    //if let Ok(state) = internal_game_init() {
    //&mut state
    //} else {
    //std::ptr::null_mut()
    //}
}

#[no_mangle]
pub extern "C" fn game_update(game_state: *mut Game_State) -> bool {
    if game_state.is_null() {
        panic!("[ FATAL ] game_update: game state is null!");
    }

    unsafe {
        let game_state = &mut *game_state;
        game_state.iter += 2;
        println!(
            "{}- {} {} ",
            game_state.iter,
            game_state.prelude,
            Instant::now()
                .duration_since(game_state.creat_time)
                .as_millis()
        );
    }
    true
}

#[no_mangle]
pub extern "C" fn game_shutdown(game_state: *mut Game_State) {
    if game_state.is_null() {
        panic!("[ FATAL ] game_shutdown: game state is null!");
    }

    unsafe {
        std::ptr::drop_in_place(game_state);
    }

    eprintln!("[ OK ] Game was shut down.");
}

//fn internal_game_init() -> Result<Game_State, Box<dyn std::error::Error>> {
//let cfg = app_config::App_Config::new(env::args());

//let mut window = gfx::window::create_render_window(&(), cfg.target_win_size, &cfg.title);
//let mut engine_state = app::create_engine_state(cfg);

//println!(
//"Working dir = {:?}\nExe = {:?}",
//engine_state.env.get_cwd(),
//engine_state.env.get_exe()
//);

//app::init_engine_systems(&mut engine_state)?;
//app::start_config_watch(&engine_state.env, &mut engine_state.config)?;

//#[cfg(debug_assertions)]
//{
//app::init_engine_debug(&mut engine_state)?;
//app::start_recording(
//&engine_state.replay_data,
//&mut engine_state.debug_systems.replay_recording_system,
//)?;
//}

//game_loop::start_game_loop(&mut engine_state, &mut window)?;

//Ok(Game_State {
//window,
//engine_state,
//})
//}
