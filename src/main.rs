#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(non_camel_case_types)]

extern crate ecs_engine;
extern crate libloading as ll;
extern crate notify;

mod game_api;
mod hotload;

use hotload::*;

const GAME_DLL_FOLDER: &str = "ecs_game/target/debug";
#[cfg(target_os = "linux")]
const GAME_DLL_FILE: &str = "libecs_game.so";
#[cfg(target_os = "windows")]
const GAME_DLL_FILE: &str = "ecs_game.dll";
#[cfg(target_os = "macos")]
const GAME_DLL_FILE: &str = "libecs_game.dylib";

#[cfg(debug_assertions)]
fn main() -> ecs_engine::core::common::Maybe_Error {
    use ecs_engine::fs::file_watcher;
    use std::path::PathBuf;
    use std::sync::mpsc::sync_channel;

    let game_dll_abs_path = format!("{}/{}", GAME_DLL_FOLDER, GAME_DLL_FILE);
    let game_dll_path = PathBuf::from(game_dll_abs_path.clone());

    // Start file watch for hotloading
    let (reload_pending_send, reload_pending_recv) = sync_channel(1);
    {
        let dll_watcher = Box::new(Game_Dll_File_Watcher::new(
            game_dll_path,
            reload_pending_send,
        ));
        let dll_watcher_cfg = file_watcher::File_Watch_Config {
            interval: std::time::Duration::from_secs(1),
            recursive_mode: notify::RecursiveMode::NonRecursive,
        };
        file_watcher::start_file_watch(
            std::path::PathBuf::from(GAME_DLL_FOLDER),
            dll_watcher_cfg,
            vec![dll_watcher],
        )?;
    }

    let (mut game_lib, mut unique_lib_path) = lib_load(&game_dll_abs_path);
    let mut game_api = unsafe { game_load(&game_lib)? };
    let game_api::Game_Bundle {
        game_state,
        game_resources,
    } = unsafe { (game_api.init)() };

    loop {
        if reload_pending_recv.try_recv().is_ok() {
            unsafe {
                (game_api.unload)(game_state, game_resources);
            }
            game_lib = lib_reload(&game_dll_abs_path, &mut unique_lib_path);
            unsafe {
                game_api = game_load(&game_lib)?;
                (game_api.reload)(game_state, game_resources);
                eprintln!(
                    "[ OK ] Reloaded API and game state from {:?}.",
                    unique_lib_path
                );
            }
        }

        unsafe {
            if !(game_api.update)(game_state, game_resources) {
                break;
            }
        }
    }

    unsafe {
        (game_api.shutdown)(game_state, game_resources);
    }

    Ok(())
}

#[cfg(not(debug_assertions))]
fn main() -> ecs_engine::core::common::Maybe_Error {
    let game_dll_abs_path = format!("{}/{}", GAME_DLL_FOLDER, GAME_DLL_FILE);
    let (game_lib, _) = lib_load(&game_dll_abs_path);
    let game_api = unsafe { game_load(&game_lib)? };
    let game_api::Game_Bundle {
        game_state,
        game_resources,
    } = unsafe { (game_api.init)() };

    loop {
        unsafe {
            if !(game_api.update)(game_state, game_resources) {
                break;
            }
        }
    }

    unsafe {
        (game_api.shutdown)(game_state, game_resources);
    }

    Ok(())
}
