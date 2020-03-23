#![allow(clippy::new_without_default)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate ecs_engine;

mod cmdline;
mod game_loop;
mod game_state;
mod gameplay_system;
mod input_utils;
mod load;
mod movement_system;
mod states;
mod systems;

#[cfg(debug_assertions)]
mod debug;

use ecs_engine::common::colors;
use ecs_engine::common::stringid::String_Id;
use ecs_engine::core::app;
use game_state::*;
use std::ffi::CStr;
use std::os::raw::c_char;

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

    if let Ok((game_state, game_resources)) = internal_game_init(&args) {
        Game_Bundle {
            game_state: Box::into_raw(game_state),
            game_resources: Box::into_raw(game_resources),
        }
    } else {
        Game_Bundle {
            game_state: std::ptr::null_mut(),
            game_resources: std::ptr::null_mut(),
        }
    }
}

/// # Safety
/// Neither pointer is allowed to be null.
#[no_mangle]
pub unsafe extern "C" fn game_update<'a>(
    game_state: *mut Game_State<'a>,
    game_resources: *mut Game_Resources<'a>,
) -> bool {
    if game_state.is_null() || game_resources.is_null() {
        fatal!("game_update: game state and/or resources are null!");
    }

    {
        let game_state = &mut *game_state;
        if game_state.engine_state.should_close {
            return false;
        }

        #[cfg(debug_assertions)]
        {
            ecs_engine::prelude::DEBUG_TRACER
                .lock()
                .unwrap()
                .start_frame();

            let log = &mut game_state.engine_state.debug_systems.log;

            if !game_state.engine_state.time.paused {
                if game_state.engine_state.time.was_paused {
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
        if game_loop::tick_game(game_state, game_resources).is_err() {
            return false;
        }
    }

    #[cfg(debug_assertions)]
    {
        let game_state = &mut *game_state;
        app::update_traces(
            &mut game_state.engine_state,
            game_state.debug_cvars.trace_overlay_refresh_rate,
        );
    }

    !(*game_state).engine_state.should_close
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
pub unsafe extern "C" fn game_unload(_game_state: *mut Game_State, _game_res: *mut Game_Resources) {
}

/// # Safety
/// Neither pointer is allowed to be null.
#[no_mangle]
pub unsafe extern "C" fn game_reload(game_state: *mut Game_State, _game_res: *mut Game_Resources) {
    #[cfg(debug_assertions)]
    {
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
}
