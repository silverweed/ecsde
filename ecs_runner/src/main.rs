#![allow(clippy::new_without_default)]
#![warn(clippy::all)]
#![allow(non_camel_case_types)]

extern crate libloading as ll;

#[cfg(debug_assertions)]
extern crate notify;

mod game_api;
mod hotload;

use hotload::*;
use std::ffi::CString;

#[cfg(debug_assertions)]
const GAME_DLL_FOLDER: &str = "target/debug";
#[cfg(not(debug_assertions))]
const GAME_DLL_FOLDER: &str = "target/release";

#[cfg(target_os = "linux")]
const GAME_DLL_FILE: &str = "libecs_game.so";
#[cfg(target_os = "windows")]
const GAME_DLL_FILE: &str = "ecs_game.dll";
#[cfg(target_os = "macos")]
const GAME_DLL_FILE: &str = "libecs_game.dylib";

macro_rules! get_argc_argv {
    ($argc: ident, $argv: ident) => {
        let c_args = ::std::env::args()
            .map(|arg| {
                CString::new(arg).unwrap_or_else(|_| {
                    println!("[ ERROR ] Failed to convert argument to CString.");
                    CString::new("").unwrap()
                })
            })
            .collect::<Vec<_>>();
        let argv = c_args.iter().map(|arg| arg.as_ptr()).collect::<Vec<_>>();
        let ($argc, $argv) = (argv.len(), argv.as_ptr());
    };
}

#[cfg(debug_assertions)]
fn main() -> std::io::Result<()> {
    use std::path::PathBuf;
    use std::sync::mpsc::sync_channel;

    let game_dll_abs_path = format!("{}/{}", GAME_DLL_FOLDER, GAME_DLL_FILE);
    let game_dll_path = PathBuf::from(game_dll_abs_path.clone());

    // Start file watch for hotloading
    #[cfg(any(target_os = "linux", feature="hotload"))]
    let (reload_pending_send, reload_pending_recv) = sync_channel(1);

    #[cfg(any(target_os = "linux", feature="hotload"))]
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

    // Convert rust args to C args, since our game API expects that.
    get_argc_argv!(argc, argv);

    let (mut game_lib, mut unique_lib_path) = lib_load(&game_dll_abs_path);
    let mut game_api = unsafe { game_load(&game_lib)? };
    let game_api::Game_Bundle {
        game_state,
        game_resources,
    } = unsafe { (game_api.init)(argv, argc) };

    loop {
        #[cfg(any(target_os = "linux", feature="hotload"))]
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

    #[cfg(any(target_os = "linux", feature="hotload"))]
    if let Err(err) = std::fs::remove_file(&unique_lib_path) {
        eprintln!(
            "[ WARNING ] Failed to remove old lib {:?}: {:?}",
            unique_lib_path, err
        );
    }

    Ok(())
}

#[cfg(not(debug_assertions))]
fn main() -> std::io::Result<()> {
    let game_dll_abs_path = format!("{}/{}", GAME_DLL_FOLDER, GAME_DLL_FILE);
    let game_lib = lib_load(&game_dll_abs_path);
    let game_api = unsafe { game_load(&game_lib)? };

    get_argc_argv!(argc, argv);

    let game_api::Game_Bundle {
        game_state,
        game_resources,
    } = unsafe { (game_api.init)(argv, argc) };

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
