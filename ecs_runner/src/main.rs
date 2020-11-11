#![allow(clippy::new_without_default)]
#![warn(clippy::all)]
#![allow(non_camel_case_types)]

mod game_api;

#[cfg(feature = "hotload")]
mod hotload;

use game_api::Game_Api;
use libloading as ll;
use std::ffi::CString;
use std::path::{Path, PathBuf};

#[cfg(feature = "hotload")]
use {hotload::*, std::sync::mpsc::Receiver};

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

#[cfg(feature = "hotload")]
fn main() -> std::io::Result<()> {
    eprintln!("Running with hotload ENABLED");

    // Convert rust args to C args, since our game API expects that.
    get_argc_argv!(argc, argv);

    let game_dll_abs_path = format!("{}/{}", GAME_DLL_FOLDER, GAME_DLL_FILE);
    let reload_pending_recv = start_hotload(PathBuf::from(game_dll_abs_path.clone()))?;

    let (mut game_lib, mut unique_lib_path) = lib_load(&game_dll_abs_path);
    let mut game_api = game_load(&game_lib)?;
    let game_api::Game_Bundle {
        game_state,
        game_resources,
    } = unsafe { (game_api.init)(argv, argc) };

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

    if let Err(err) = std::fs::remove_file(&unique_lib_path) {
        eprintln!(
            "[ WARNING ] Failed to remove old lib {:?}: {:?}",
            unique_lib_path, err
        );
    }

    Ok(())
}

#[cfg(not(feature = "hotload"))]
fn main() -> std::io::Result<()> {
    eprintln!("Running with hotload DISABLED");

    // Convert rust args to C args, since our game API expects that.
    get_argc_argv!(argc, argv);

    let game_dll_abs_path = format!("{}/{}", GAME_DLL_FOLDER, GAME_DLL_FILE);
    let (game_lib, _) = lib_load(&game_dll_abs_path);
    let game_api = game_load(&game_lib)?;
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

fn game_load(game_lib: &ll::Library) -> ll::Result<Game_Api<'_>> {
    unsafe {
        Ok(Game_Api {
            init: game_lib.get(b"game_init\0")?,
            update: game_lib.get(b"game_update\0")?,
            shutdown: game_lib.get(b"game_shutdown\0")?,
            #[cfg(debug_assertions)]
            unload: game_lib.get(b"game_unload\0")?,
            #[cfg(debug_assertions)]
            reload: game_lib.get(b"game_reload\0")?,
        })
    }
}

fn lib_load(lib_path: &str) -> (ll::Library, PathBuf) {
    let loaded_path = if cfg!(feature = "hotload") {
        let unique_name = {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let index = lib_path.rfind('.').unwrap();
            let (before, after) = lib_path.split_at(index);
            format!("{}-{}{}", before, timestamp, after)
        };
        std::fs::copy(&lib_path, &unique_name)
            .expect("[ ERROR ] Failed to copy lib to unique path");
        Path::new(&unique_name).canonicalize().unwrap()
    } else {
        PathBuf::from(lib_path)
    };

    eprintln!("[ INFO ] Loading lib {:?}", loaded_path);

    (
        ll::Library::new(loaded_path.as_os_str()).unwrap_or_else(|err| {
            panic!(
                "[ ERROR ] Failed to load library '{:?}': {:?}",
                loaded_path, err
            )
        }),
        loaded_path,
    )
}

#[cfg(feature = "hotload")]
fn start_hotload(game_dll_path: PathBuf) -> std::io::Result<Receiver<()>> {
    use std::sync::mpsc::sync_channel;

    let (reload_pending_send, reload_pending_recv) = sync_channel(1);

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

    Ok(reload_pending_recv)
}
