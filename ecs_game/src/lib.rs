#![allow(clippy::new_without_default)]
#![allow(non_camel_case_types)]
#![allow(clippy::too_many_arguments)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate inle_diagnostics;

#[macro_use]
extern crate inle_common;

#[macro_use]
extern crate inle_math;

#[macro_use]
extern crate inle_ecs;

#[cfg(debug_assertions)]
#[macro_use]
extern crate lazy_static;

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

use game_state::*;
use inle_core::{sleep, time};
use std::convert::TryInto;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::time::{Duration, Instant};

#[cfg(debug_assertions)]
use {inle_app::app, inle_common::colors};

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
    let mut loggers = inle_diagnostics::log::create_loggers();
    inle_diagnostics::log::add_default_logger(&mut loggers);

    linfo!("Initializing game...");

    // Copy all arguments into rust strings
    let mut args: Vec<String> = Vec::with_capacity(args_count);
    for i in 0..args_count {
        let arg = raw_args.add(i);
        assert!(!(*arg).is_null(), "{}-th cmdline argument is null!", i);
        args.push(new_string_from_c_char_ptr(*arg));
    }

    match internal_game_init(&args, loggers) {
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

    inle_gfx::render_window::start_new_frame(&mut game_state.window);

    #[cfg(debug_assertions)]
    {
        inle_diagnostics::prelude::DEBUG_TRACERS
            .lock()
            .unwrap()
            .values_mut()
            .for_each(|t| t.lock().unwrap().start_frame());

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
        // We also add hints every X frames to catch also all functions
        // that weren't added during the very first frame.
        // This is done asynchronously since it's a quite heavy operation (needs a sort + dedup)
        if game_state.update_trace_hints_countdown == 0 {
            let console = game_state.engine_state.debug_systems.console.clone();
            let tracers = inle_diagnostics::prelude::DEBUG_TRACERS.lock().unwrap();
            let saved_traces = tracers
                .iter()
                .map(|(_, tracer)| tracer.lock().unwrap().saved_traces.to_vec())
                .flatten()
                .collect::<Vec<_>>();
            game_state
                .engine_state
                .systems
                .long_task_mgr
                .create_task(move || {
                    use std::collections::HashSet;

                    let fn_names: HashSet<_> = saved_traces
                        .into_iter()
                        .map(|trace| String::from(trace.info.tag))
                        .collect();
                    // Note: we do the heavy work in the task and not in add_hints so we can lock the console
                    // for as little as possible.
                    let cur_hints = console
                        .lock()
                        .unwrap()
                        .hints
                        .get("trace")
                        .cloned()
                        .unwrap_or_default();
                    let cur_hints: HashSet<String> = cur_hints.into_iter().collect();
                    let fn_names = fn_names.difference(&cur_hints).cloned();
                    console.lock().unwrap().add_hints("trace", fn_names);
                });
            game_state.update_trace_hints_countdown = 60;
        } else {
            game_state.update_trace_hints_countdown -= 1;
        }

        app::update_traces(
            &mut game_state.engine_state,
            game_state.debug_cvars.trace_overlay_refresh_rate,
        );

        let print_draw_stats = game_state
            .debug_cvars
            .print_draw_stats
            .read(&game_state.engine_state.config);
        if print_draw_stats && game_state.engine_state.cur_frame % 100 == 0 {
            ldebug!(
                "Draw calls this frame: {}. Time taken: {:?}",
                game_state.window.gl.n_draw_calls_this_frame,
                t_before_work.elapsed()
            );
        }
    }

    game_state.engine_state.frame_alloc.dealloc_all();

    ///// !!! Must not use frame_alloc after this !!! /////

    if !inle_win::window::has_vsync(&game_state.window) {
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
            lverbose!(
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

    (*game_state)
        .engine_state
        .systems
        .long_task_mgr
        .begin_shutdown();

    #[cfg(debug_assertions)]
    {
        use inle_debug::console::save_console_hist;
        let engine_state = &(*game_state).engine_state;
        save_console_hist(
            &engine_state.debug_systems.console.lock().unwrap(),
            &engine_state.env,
        )
        .unwrap_or_else(|err| lwarn!("Failed to save console history: {}", err));
    }

    inle_gfx::render_window::shutdown(&mut (*game_state).window);
    (*game_state)
        .engine_state
        .systems
        .long_task_mgr
        .block_until_shutdown_complete();

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
    inle_diagnostics::log::unregister_loggers();
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

    inle_diagnostics::log::register_loggers(&game_state.engine_state.loggers);

    game_state
        .engine_state
        .debug_systems
        .debug_ui
        .get_overlay(sid!("msg"))
        .add_line("+++ GAME RELOADED +++")
        .with_color(colors::rgb(255, 128, 0));
    ldebug!("+++ GAME RELOADED +++");

    inle_win::window::recreate_window(&mut game_state.window);
    inle_gfx::render_window::recreate_render_window(&mut game_state.window);
}
