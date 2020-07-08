#![allow(clippy::new_without_default)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate ecs_engine;

mod cmdline;
mod collisions;
mod directions;
mod entities;
mod game_loop;
mod game_state;
mod gameplay_system;
mod gfx;
mod input_utils;
mod levels;
mod load;
mod movement_system;
mod spatial;
mod states;
mod systems;

#[cfg(debug_assertions)]
mod debug;

use ecs_engine::common::colors;
use ecs_engine::common::stringid::String_Id;
use ecs_engine::core::{app, sleep, time};
use ecs_engine::gfx as ngfx;
use game_state::*;
use std::convert::TryInto;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::time::{Duration, Instant};

/// Given a c_char pointer, returns a String allocated from the raw string it points to,
/// or an empty string if the conversion fails.
fn new_string_from_c_char_ptr(c_char_ptr: *const c_char) -> String {
    let cstr = unsafe { CStr::from_ptr(c_char_ptr) };
    let str_slice = cstr.to_str().unwrap_or_else(|_| {
        lerr!("Failed to convert argument {:?} to a valid String.", cstr);
        ""
    });
    String::from(str_slice)
}

/////////////////////////////////////////////////////////////////////////////
//                        FOREIGN FUNCTION API                             //
/////////////////////////////////////////////////////////////////////////////

// Note: the lifetime is actually ignored. The Game_State/Resources's lifetime management is manual
// and it's performed by the game runner (the Game_State/Resources stay alive from game_init()
// to game_shutdown()).
/// `raw_args` is a pointer to the first command-line argument given to the game runner,
/// `args_count` is the total number of arguments.
/// # Safety
/// If `args_count` > 0, `raw_args` must point to valid memory.
#[no_mangle]
pub unsafe extern "C" fn game_init<'a>(
    raw_args: *const *const c_char,
    args_count: usize,
) -> Game_Bundle<'a> {
    linfo!("Initializing game...");

    // Copy all arguments into rust strings
    let mut args: Vec<String> = Vec::with_capacity(args_count);
    for i in 0..args_count {
        let arg = raw_args.add(i);
        assert!(!(*arg).is_null(), "{}-th cmdline argument is null!", i);
        args.push(new_string_from_c_char_ptr(*arg));
    }

    match internal_game_init(&args) {
        Ok((game_state, game_resources)) => Game_Bundle {
            game_state: Box::into_raw(game_state),
            game_resources: Box::into_raw(game_resources),
        },
        Err(err) => {
            lerr!("internal_game_init() failed with err {}", err);
            Game_Bundle {
                game_state: std::ptr::null_mut(),
                game_resources: std::ptr::null_mut(),
            }
        }
    }
}

/// # Safety
/// Neither pointer is allowed to be null.
#[no_mangle]
pub unsafe extern "C" fn game_update<'s, 'r>(
    game_state: *mut Game_State<'s>,
    game_resources: *mut Game_Resources<'r>,
) -> bool
where
    'r: 's,
{
    if game_state.is_null() || game_resources.is_null() {
        fatal!("game_update: game state and/or resources are null!");
    }

    let game_state = &mut *game_state;
    if game_state.engine_state.should_close {
        return false;
    }

    let t_before_work = Instant::now();

    #[cfg(debug_assertions)]
    {
        ecs_engine::prelude::DEBUG_TRACER
            .lock()
            .unwrap()
            .start_frame();

        let log = &mut game_state.engine_state.debug_systems.log;

        if !game_state.engine_state.time.paused {
            if game_state.engine_state.time.was_paused() {
                // Just resumed
                game_state
                    .engine_state
                    .debug_systems
                    .debug_ui
                    .frame_scroller
                    .manually_selected = false;
                log.reset_from_frame(game_state.engine_state.cur_frame);
            }
            log.start_frame();
        }
    }

    let game_resources = &mut *game_resources;

    let target_time_per_frame = Duration::from_micros(
        (game_state
            .cvars
            .gameplay_update_tick_ms
            .read(&game_state.engine_state.config)
            * 1000.0) as u64,
    );

    if game_loop::tick_game(game_state, game_resources).is_err() {
        return false;
    }

    #[cfg(debug_assertions)]
    {
        // Initialize the hints for the `trace` command. Do this after the first
        // frame so the tracer contains all the function names.
        static mut INIT_CONSOLE_FN_NAME_HINTS: std::sync::Once = std::sync::Once::new();
        INIT_CONSOLE_FN_NAME_HINTS.call_once(|| {
            let console = &mut game_state.engine_state.debug_systems.console;
            let fn_names: std::collections::HashSet<_> = {
                let tracer = ecs_engine::prelude::DEBUG_TRACER.lock().unwrap();
                tracer
                    .saved_traces
                    .iter()
                    .map(|trace| trace.info.tag)
                    .collect()
            };
            console.add_hints("trace", fn_names.into_iter().map(String::from));
        });

        app::update_traces(
            &mut game_state.engine_state,
            game_state.debug_cvars.trace_overlay_refresh_rate,
        );
    }

    game_state.engine_state.frame_alloc.dealloc_all();

    ///// !!! Must not use frame_alloc after this !!! /////

    if !ngfx::window::has_vsync(&game_state.window) {
        let mut t_elapsed_for_work = t_before_work.elapsed();
        if t_elapsed_for_work < target_time_per_frame {
            while t_elapsed_for_work < target_time_per_frame {
                if let Some(granularity) = game_state.sleep_granularity {
                    if granularity < target_time_per_frame - t_elapsed_for_work {
                        let gra_ns = granularity.as_nanos();
                        let rem_ns = (target_time_per_frame - t_elapsed_for_work).as_nanos();
                        let time_to_sleep =
                            Duration::from_nanos((rem_ns / gra_ns).try_into().unwrap());
                        sleep::sleep(time_to_sleep);
                    }
                }

                t_elapsed_for_work = t_before_work.elapsed();
            }
        } else {
            lerr!(
                "Frame budget exceeded! At frame {}: {} / {} ms",
                game_state.engine_state.cur_frame,
                time::to_ms_frac(&t_elapsed_for_work),
                time::to_ms_frac(&target_time_per_frame)
            );
        }
    }

    #[cfg(debug_assertions)]
    {
        game_state.engine_state.prev_frame_time = t_before_work.elapsed();
    }

    !game_state.engine_state.should_close
}

/// # Safety
/// Neither pointer is allowed to be null.
/// After calling this function, both pointers become invalid and must not be used anymore.
#[no_mangle]
pub unsafe extern "C" fn game_shutdown(
    game_state: *mut Game_State,
    game_resources: *mut Game_Resources,
) {
    use std::alloc::{dealloc, Layout};

    if game_state.is_null() || game_resources.is_null() {
        fatal!("game_shutdown: game state and/or resources are null!");
    }

    #[cfg(debug_assertions)]
    {
        use ecs_engine::debug::console::save_console_hist;
        let engine_state = &(*game_state).engine_state;
        save_console_hist(&engine_state.debug_systems.console, &engine_state.env)
            .unwrap_or_else(|err| lwarn!("Failed to save console history: {}", err));
    }

    std::ptr::drop_in_place(game_state);
    dealloc(game_state as *mut u8, Layout::new::<Game_State>());

    std::ptr::drop_in_place(game_resources);
    dealloc(game_resources as *mut u8, Layout::new::<Game_Resources>());

    lok!("Game was shut down.");
}

/// # Safety
/// Neither pointer is allowed to be null.
#[no_mangle]
#[cfg(debug_assertions)]
pub unsafe extern "C" fn game_unload(_game_state: *mut Game_State, _game_res: *mut Game_Resources) {
}

/// # Safety
/// Neither pointer is allowed to be null.
#[no_mangle]
#[cfg(debug_assertions)]
pub unsafe extern "C" fn game_reload(game_state: *mut Game_State, _game_res: *mut Game_Resources) {
    if game_state.is_null() {
        fatal!("game_reload: game state is null!");
    }

    let game_state = &mut *game_state;
    game_state
        .engine_state
        .debug_systems
        .debug_ui
        .get_fadeout_overlay(String_Id::from("msg"))
        .add_line_color("+++ GAME RELOADED +++", colors::rgb(255, 128, 0));
}
