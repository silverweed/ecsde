#![allow(non_camel_case_types)]

#[macro_use]
extern crate inle_diagnostics;

#[macro_use]
extern crate inle_common;

#[macro_use]
extern crate inle_math;

mod game;
mod input;
mod phases;

#[cfg(debug_assertions)]
mod debug;

use std::ffi::c_char;

type Phase_Manager = inle_app::phases::Phase_Manager<phases::Phase_Args>;

pub struct Game_State {
    should_quit: bool,
    env: inle_core::env::Env_Info,
    config: inle_cfg::config::Config,
    app_config: inle_app::app_config::App_Config,

    loggers: inle_diagnostics::log::Loggers,

    rng: inle_core::rand::Default_Rng,

    time: inle_core::time::Time,
    cur_frame: u64,
    prev_frame_time: std::time::Duration,

    frame_alloc: inle_alloc::temp::Temp_Allocator,

    window: inle_gfx::render_window::Render_Window_Handle,
    batches: inle_gfx::render::batcher::Batches,
    lights: inle_gfx::light::Lights,

    input: inle_input::input_state::Input_State,

    default_font: inle_resources::gfx::Font_Handle,

    engine_cvars: inle_app::app::Engine_CVars,

    ui: inle_ui::Ui_Context,

    phase_mgr: Phase_Manager,

    #[cfg(debug_assertions)]
    debug_systems: inle_app::debug_systems::Debug_Systems,

    #[cfg(debug_assertions)]
    fps_counter: inle_debug::fps::Fps_Counter,
}

pub struct Game_Resources {
    pub gfx: inle_resources::gfx::Gfx_Resources,
    pub audio: inle_resources::audio::Audio_Resources,
    pub shader_cache: inle_resources::gfx::Shader_Cache,
}

#[repr(C)]
pub struct Game_Bundle {
    pub game_state: *mut Game_State,
    pub game_resources: *mut Game_Resources,
}

// # Safety
// args should not be null and should contain a number of strings consistent with args_count
#[no_mangle]
pub unsafe extern "C" fn game_init(_args: *const *const c_char, _args_count: usize) -> Game_Bundle {
    let mut game_res = game::create_game_resources();
    let mut game_state = game::internal_game_init();

    game::game_post_init(&mut game_state, &mut game_res);

    Game_Bundle {
        game_state: Box::into_raw(game_state),
        game_resources: Box::into_raw(game_res),
    }
}

// # Safety
// game_state and game_res must be non-null
#[no_mangle]
pub unsafe extern "C" fn game_update(
    game_state: *mut Game_State,
    game_res: *mut Game_Resources,
) -> bool {
    let game_state = &mut *game_state;
    let game_res = &mut *game_res;

    let t_before_work = std::time::Instant::now();
    {
        trace!("game_update");

        game::start_frame(game_state);

        //
        // Input
        //
        game::process_input(game_state, game_res);

        //
        // Update
        //
        game::update(game_state, game_res);

        #[cfg(debug_assertions)]
        {
            debug::update_debug(game_state, game_res);
        }

        //
        // Render
        //
        game::render(game_state, game_res);
    }

    game_state.prev_frame_time = t_before_work.elapsed();

    game::end_frame(game_state);

    !game_state.should_quit
}

// # Safety
// game_state and game_res must be non-null
#[no_mangle]
pub unsafe extern "C" fn game_shutdown(game_state: *mut Game_State, game_res: *mut Game_Resources) {
    inle_gfx::render::batcher::clear_batches(&mut (*game_state).batches);
    inle_gfx::render_window::shutdown(&mut (*game_state).window);

    std::ptr::drop_in_place(game_state);
    std::alloc::dealloc(
        game_state as *mut u8,
        std::alloc::Layout::new::<Game_State>(),
    );

    std::ptr::drop_in_place(game_res);
    std::alloc::dealloc(
        game_res as *mut u8,
        std::alloc::Layout::new::<Game_Resources>(),
    );
}

// # Safety
// game_state and game_res must be non-null
#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn game_unload(_game_state: *mut Game_State, _game_res: *mut Game_Resources) {
    inle_diagnostics::log::unregister_loggers();
}

// # Safety
// game_state and game_res must be non-null
#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn game_reload(game_state: *mut Game_State, _game_res: *mut Game_Resources) {
    let game_state = &mut *game_state;

    inle_diagnostics::log::register_loggers(&game_state.loggers);

    game_state
        .debug_systems
        .debug_ui
        .get_overlay(sid!("msg"))
        .add_line("+++ GAME RELOADED +++")
        .with_color(inle_common::colors::rgb(255, 128, 0));
    ldebug!("+++ GAME RELOADED +++");

    inle_win::window::recreate_window(&mut game_state.window);
    inle_gfx::render_window::recreate_render_window(&mut game_state.window);
}
