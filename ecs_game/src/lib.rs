#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

extern crate anymap;
extern crate cgmath;
extern crate ecs_engine;
#[cfg(test)]
extern crate float_cmp;

mod controllable_system;
mod ecs;
mod game_loop;
mod gameplay_system;
mod gfx;
mod scene_tree;
mod states;

use ecs_engine::cfg::Cfg_Var;
use ecs_engine::core::common::{colors, rand};
use ecs_engine::core::{app, app_config};
use ecs_engine::debug;
use ecs_engine::gfx::{self as ngfx, window};
use ecs_engine::input;
use std::env;
use std::time::Duration;

#[repr(C)]
pub struct Game_State<'a> {
    pub window: window::Window_Handle,
    pub engine_state: app::Engine_State<'a>,

    pub render_system: gfx::render_system::Render_System,
    pub gameplay_system: gameplay_system::Gameplay_System,

    //pub state_mgr: states::state_manager::State_Manager,
    #[cfg(debug_assertions)]
    pub fps_debug: debug::fps::Fps_Console_Printer,

    pub execution_time: Duration,
    pub input_provider: Box<dyn input::provider::Input_Provider>,
    pub is_replaying: bool,

    //// Cfg vars
    pub update_time: Cfg_Var<i32>,
    pub smooth_by_extrapolating_velocity: Cfg_Var<bool>,
    #[cfg(debug_assertions)]
    pub extra_frame_sleep_ms: Cfg_Var<i32>,
    #[cfg(debug_assertions)]
    pub record_replay: Cfg_Var<bool>,

    pub rng: rand::Default_Rng,
}

/////////////////////////////////////////////////////////////////////////////
//                        FOREIGN FUNCTION API                             //
/////////////////////////////////////////////////////////////////////////////

// Note: the lifetime is actually ignored. The Game_State's lifetime management is manual
// and it's performed by the game runner (the Game_State stays alive from game_init()
// to game_shutdown()).
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
    if game_state.engine_state.should_close {
        return false;
    }

    if let Ok(true) = game_loop::tick_game(game_state) {
        // All green
    } else {
        return false;
    }

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

#[no_mangle]
pub unsafe extern "C" fn game_unload(_game_state: *mut Game_State) {}

#[no_mangle]
pub unsafe extern "C" fn game_reload(game_state: *mut Game_State) {
    use ecs_engine::core::common::stringid::String_Id;

    if game_state.is_null() {
        panic!("[ FATAL ] game_reload: game state is null!");
    }

    let game_state = &mut *game_state;
    game_state
        .engine_state
        .debug_systems
        .debug_ui_system
        .get_fadeout_overlay(String_Id::from("msg"))
        .add_line_color("+++ GAME RELOADED +++", colors::rgb(255, 128, 0));
}

/////////////////////////////////////////////////////////////////////////////
//                      END FOREIGN FUNCTION API                           //
/////////////////////////////////////////////////////////////////////////////

fn internal_game_init<'a>() -> Result<Box<Game_State<'a>>, Box<dyn std::error::Error>> {
    let mut game_state = create_game_state()?;
    let env = &game_state.engine_state.env;
    let gres = &mut game_state.engine_state.gfx_resources;
    let cfg = &game_state.engine_state.config;

    game_state
        .gameplay_system
        .init(gres, env, &mut game_state.rng, cfg)?;
    game_state
        .render_system
        .init(gfx::render_system::Render_System_Config {
            clear_color: colors::rgb(22, 0, 22),
        })?;
    //init_states(&mut game_state.state_mgr, &mut game_state.engine_state)?;

    Ok(game_state)
}

fn create_game_state<'a>() -> Result<Box<Game_State<'a>>, Box<dyn std::error::Error>> {
    let cfg = app_config::App_Config::new(env::args());

    let window = ngfx::window::create_render_window(&(), cfg.target_win_size, &cfg.title);
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
        app::start_recording(&mut engine_state)?;
    }

    let cfg = &engine_state.config;
    let input_provider = app::create_input_provider(&mut engine_state.replay_data, cfg);
    let is_replaying = !input_provider.is_realtime_player_input();
    let update_time = Cfg_Var::new("engine/gameplay/gameplay_update_tick_ms", cfg);
    let smooth_by_extrapolating_velocity =
        Cfg_Var::new("engine/rendering/smooth_by_extrapolating_velocity", cfg);
    #[cfg(debug_assertions)]
    let extra_frame_sleep_ms = Cfg_Var::new("engine/debug/extra_frame_sleep_ms", cfg);
    #[cfg(debug_assertions)]
    let record_replay = Cfg_Var::new("engine/debug/replay/record", cfg);

    Ok(Box::new(Game_State {
        window,
        engine_state,
        #[cfg(debug_assertions)]
        fps_debug: debug::fps::Fps_Console_Printer::new(&Duration::from_secs(2), "game"),
        execution_time: Duration::default(),
        update_time,
        smooth_by_extrapolating_velocity,
        #[cfg(debug_assertions)]
        extra_frame_sleep_ms,
        #[cfg(debug_assertions)]
        record_replay,
        input_provider,
        is_replaying,
        render_system: gfx::render_system::Render_System::new(),
        gameplay_system: gameplay_system::Gameplay_System::new(),
        //state_mgr: states::state_manager::State_Manager::new(),
        rng: rand::new_rng()?,
    }))
}

//fn init_states(
//state_mgr: &mut states::state_manager::State_Manager,
//engine_state: &mut app::Engine_State,
//) -> Maybe_Error {
//let base_state = Box::new(states::persistent::game_base_state::Game_Base_State {});
//state_mgr.add_persistent_state(engine_state, base_state);
//#[cfg(debug_assertions)]
//{
//let debug_base_state =
//Box::new(states::persistent::debug_base_state::Debug_Base_State::new());
//state_mgr.add_persistent_state(engine_state, debug_base_state);
//}
//Ok(())
//}
