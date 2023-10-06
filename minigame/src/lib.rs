#![allow(non_camel_case_types)]

#[macro_use]
extern crate inle_diagnostics;

#[macro_use]
extern crate inle_common;

#[macro_use]
extern crate inle_math;

#[macro_use]
extern crate smallvec;

mod entity;
mod game;
mod input;
mod phases;
mod sprites;

#[cfg(debug_assertions)]
mod debug;

pub use game::{Game_Resources, Game_State};
use std::ffi::c_char;
use std::time::Duration;

#[repr(C)]
pub struct Game_Bundle {
    pub game_state: *mut Game_State,
    pub game_resources: *mut Game_Resources,
}

/// # Safety
/// args should not be null and should contain a number of strings consistent with args_count
#[no_mangle]
pub unsafe extern "C" fn game_init(args: *const *const c_char, args_count: usize) -> Game_Bundle {
    let args = inle_app::app::args_to_string_vec(args, args_count);
    let args = game::parse_game_args(&args);
    let mut game_res = game::create_game_resources();
    let mut game_state = game::internal_game_init(&args);

    game::game_post_init(&mut game_state, &mut game_res, &args);

    Game_Bundle {
        game_state: Box::into_raw(game_state),
        game_resources: Box::into_raw(game_res),
    }
}

/// # Safety
/// game_state and game_res must be non-null
#[no_mangle]
pub unsafe extern "C" fn game_update(
    game_state: *mut Game_State,
    game_res: *mut Game_Resources,
) -> bool {
    let game_state = &mut *game_state;
    let game_res = &mut *game_res;

    let t_before_work = std::time::Instant::now();

    inle_diagnostics::prelude::DEBUG_TRACERS
        .lock()
        .unwrap()
        .values_mut()
        .for_each(|t| t.lock().unwrap().start_frame());

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

    if !inle_win::window::has_vsync(&game_state.window) {
        let target_time_per_frame = Duration::from_micros(
            (game_state
                .engine_cvars
                .gameplay_update_tick_ms
                .read(&game_state.config)
                * 1000.0) as u64,
        );
        inle_app::app::limit_framerate(
            t_before_work,
            target_time_per_frame,
            game_state.sleep_granularity,
            game_state.cur_frame,
        );
    }

    !game_state.should_quit
}

/// # Safety
/// game_state and game_res must be non-null
#[no_mangle]
pub unsafe extern "C" fn game_shutdown(game_state: *mut Game_State, game_res: *mut Game_Resources) {
    let gs = &mut *game_state;
    #[cfg(debug_assertions)]
    {
        inle_debug::console::save_console_hist(&gs.debug_systems.console.lock().unwrap(), &gs.env)
            .unwrap_or_else(|err| lwarn!("Failed to save console history: {}", err));
    }

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

/// # Safety
/// game_state and game_res must be non-null
#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn game_unload(_game_state: *mut Game_State, _game_res: *mut Game_Resources) {
    inle_diagnostics::log::unregister_loggers();
}

/// # Safety
/// game_state and game_res must be non-null
#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn game_reload(game_state: *mut Game_State, game_res: *mut Game_Resources) {
    let game_state = &mut *game_state;
    let game_res = &mut *game_res;

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

    // Recreate phases (otherwise they'll not be hot reloaded properly, I guess due to them
    // being trait objects or something)
    let cur_phase_stack = game_state.phase_mgr.current_phase_stack().to_vec();
    let mut phase_args = phases::Phase_Args::new(game_state, game_res);
    game_state.phase_mgr.teardown(&mut phase_args);

    game::register_game_phases(game_state, game_res);

    for phase_id in cur_phase_stack {
        game_state.phase_mgr.push_phase(phase_id, &mut phase_args);
    }
}
