#![allow(clippy::new_without_default)]
#![warn(clippy::all)]
#![allow(non_camel_case_types)]

mod game_api;
#[cfg(feature = "hotload")]
mod hotload;

use game_api::Game_Api;
#[cfg(feature = "hotload")]
use hotload::*;
use libloading as ll;
use std::ffi::{c_char, CString};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;

#[cfg(target_os = "linux")]
const GAME_DLL_FILE: &str = "libminigame.so";
#[cfg(target_os = "windows")]
const GAME_DLL_FILE: &str = "minigame.dll";
#[cfg(target_os = "macos")]
const GAME_DLL_FILE: &str = "libminigame.dylib";

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

fn main() -> io::Result<()> {
    // Convert rust args to C args, since our game API expects that.
    get_argc_argv!(argc, argv);

    // Look for game DLL in the same path as the runner
    let mut game_dll_abs_path = std::fs::canonicalize(std::env::current_exe()?)?;
    game_dll_abs_path.pop();
    game_dll_abs_path.push(GAME_DLL_FILE);

    let create_temp = cfg!(feature = "hotload");
    match lib_load(&game_dll_abs_path, create_temp) {
        Ok(res) => {
            let game_lib = res.lib;
            let unique_lib_path = res.path.clone();
            match game_load(&game_lib) {
                Ok(game_api) => {
                    if let Err(main_err) = {
                        if game_api.unload.is_some() && game_api.reload.is_some() {
                            main_with_hotload(
                                argc,
                                argv,
                                &game_dll_abs_path,
                                game_lib,
                                unique_lib_path,
                            )
                        } else {
                            main_without_hotload(argc, argv, game_lib)
                        }
                    } {
                        if create_temp {
                            if let Err(rm_err) = std::fs::remove_file(&res.path) {
                                eprintln!(
                                    "[ WARNING ] Failed to remove old lib {:?}: {:?}",
                                    res.path, rm_err
                                );
                            }
                        }
                        Err(main_err)
                    } else {
                        Ok(())
                    }
                }
                Err(game_load_err) => {
                    if create_temp {
                        let _err = game_lib.close();
                        if let Err(rm_err) = std::fs::remove_file(&res.path) {
                            eprintln!(
                                "[ WARNING ] Failed to remove old lib {:?}: {:?}",
                                res.path, rm_err
                            );
                        }
                    }
                    Err(io::Error::new(io::ErrorKind::Other, game_load_err))
                }
            }
        }
        Err(lib_load_err) => {
            if create_temp {
                if let Err(rm_err) = std::fs::remove_file(&lib_load_err.path) {
                    eprintln!(
                        "[ WARNING ] Failed to remove old lib {:?}: {:?}",
                        lib_load_err.path, rm_err
                    );
                }
            }
            panic!(
                "[ ERROR ] Failed to load library '{:?}': {:?}",
                lib_load_err.path, lib_load_err.err
            );
        }
    }
}

#[cfg(feature = "hotload")]
fn main_with_hotload(
    argc: usize,
    argv: *const *const c_char,
    game_dll_abs_path: &Path,
    mut game_lib: ll::Library,
    mut unique_lib_path: PathBuf,
) -> io::Result<()> {
    eprintln!("[ INFO ] Running with hotload ENABLED");

    let mut game_api = game_load(&game_lib).unwrap();
    let game_api::Game_Bundle {
        game_state,
        game_resources,
    } = unsafe { (game_api.init)(argv, argc) };

    let reload_pending_recv = start_hotload(game_dll_abs_path.to_path_buf())?;

    loop {
        if reload_pending_recv.try_recv().is_ok() {
            unsafe {
                (game_api.unload.unwrap())(game_state, game_resources);
            }
            game_lib = lib_reload(game_dll_abs_path, &mut unique_lib_path);
            unsafe {
                game_api = game_load(&game_lib).unwrap();
                (game_api.reload.unwrap())(game_state, game_resources);
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

    game_lib
        .close()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    if let Err(err) = std::fs::remove_file(&unique_lib_path) {
        eprintln!(
            "[ WARNING ] Failed to remove old lib {:?}: {:?}",
            unique_lib_path, err
        );
    }

    Ok(())
}

#[cfg(not(feature = "hotload"))]
fn main_with_hotload(
    argc: usize,
    argv: *const *const c_char,
    _game_dll_abs_path: &Path,
    game_lib: ll::Library,
    _unique_lib_path: PathBuf,
) -> io::Result<()> {
    main_without_hotload(argc, argv, game_lib)
}

fn main_without_hotload(
    argc: usize,
    argv: *const *const c_char,
    game_lib: ll::Library,
) -> io::Result<()> {
    eprintln!("[ INFO ] Running with hotload DISABLED");

    let game_api = game_load(&game_lib).unwrap();
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

    game_lib
        .close()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
}

fn game_load(game_lib: &ll::Library) -> Result<Game_Api<'_>, ll::Error> {
    unsafe {
        let init = game_lib.get(b"game_init\0")?;
        let update = game_lib.get(b"game_update\0")?;
        let shutdown = game_lib.get(b"game_shutdown\0")?;
        let unload = game_lib.get(b"game_unload\0").ok();
        let reload = game_lib.get(b"game_reload\0").ok();
        Ok(Game_Api {
            init,
            update,
            shutdown,
            unload,
            reload,
        })
    }
}

struct Lib_Load_Res {
    pub lib: ll::Library,
    pub path: PathBuf,
}

struct Lib_Load_Err {
    pub err: ll::Error,
    pub path: PathBuf,
}

impl std::fmt::Debug for Lib_Load_Err {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{:?}", self.err)
    }
}

fn lib_load(lib_path: &Path, create_temp: bool) -> Result<Lib_Load_Res, Lib_Load_Err> {
    eprintln!("[ INFO ] Game lib is {}", lib_path.display());

    let loaded_path = if create_temp {
        let unique_name = {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let path = lib_path.as_os_str().to_str().unwrap();
            let index = path.rfind('.').unwrap();
            let (before, after) = path.split_at(index);
            format!("{}-{}{}", before, timestamp, after)
        };
        std::fs::copy(&lib_path, &unique_name)
            .expect("[ ERROR ] Failed to copy lib to unique path");
        Path::new(&unique_name).canonicalize().unwrap()
    } else {
        PathBuf::from(lib_path)
    };

    eprintln!("[ INFO ] Loading lib {}", loaded_path.display());

    unsafe { ll::Library::new(loaded_path.as_os_str()) }
        .map_err(|err| Lib_Load_Err {
            err,
            path: loaded_path.clone(),
        })
        .map(|lib| Lib_Load_Res {
            lib,
            path: loaded_path,
        })
}

#[cfg(feature = "hotload")]
fn start_hotload(game_dll_path: PathBuf) -> io::Result<Receiver<()>> {
    use std::sync::mpsc::sync_channel;

    let (reload_pending_send, reload_pending_recv) = sync_channel(1);

    let dll_watcher = Box::new(Game_Dll_File_Watcher::new(
        game_dll_path.clone(),
        reload_pending_send,
    ));
    let dll_watcher_cfg = file_watcher::File_Watch_Config {
        interval: std::time::Duration::from_secs(1),
    };
    let dll_folder = game_dll_path.parent().unwrap();
    file_watcher::start_file_watch(dll_folder.to_path_buf(), dll_watcher_cfg, vec![dll_watcher])?;

    Ok(reload_pending_recv)
}
