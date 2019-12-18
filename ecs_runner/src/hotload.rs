#[cfg(debug_assertions)]
pub mod file_watcher;

use super::game_api::Game_Api;
use std::path::{Path, PathBuf};

#[cfg(debug_assertions)]
use notify::DebouncedEvent;
#[cfg(debug_assertions)]
use std::sync::mpsc::SyncSender;

#[cfg(debug_assertions)]
pub struct Game_Dll_File_Watcher {
    file: PathBuf,
    pub reload_pending: SyncSender<()>,
}

#[cfg(debug_assertions)]
impl Game_Dll_File_Watcher {
    pub fn new(file: PathBuf, reload_pending: SyncSender<()>) -> Self {
        Game_Dll_File_Watcher {
            file: file.canonicalize().unwrap(),
            reload_pending,
        }
    }
}

#[cfg(debug_assertions)]
impl file_watcher::File_Watcher_Event_Handler for Game_Dll_File_Watcher {
    fn handle(&mut self, event: &DebouncedEvent) {
        eprintln!("EVENT = {:?}", event);
        match event {
            DebouncedEvent::Write(path)
            | DebouncedEvent::Create(path)
            | DebouncedEvent::Chmod(path)
            | DebouncedEvent::Remove(path) => match path.canonicalize() {
                Ok(canon_path) => {
                    if canon_path == self.file {
                        let _ = self.reload_pending.try_send(());
                    } else {
                        eprintln!("Not reloading because path != self.file.\n  path = {:?}\n  file = {:?}", canon_path, self.file);
                    }
                }
                Err(err) => {
                    eprintln!("Failed to canonicalize path {:?}: {:?}", path, err);
                }
            },
            _ => (),
        }
    }
}

pub fn lib_load(lib_path: &str) -> (ll::Library, PathBuf) {
    let unique_name = {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let index = lib_path.rfind('.').unwrap();
        let (before, after) = lib_path.split_at(index);
        format!("{}-{}{}", before, timestamp, after)
    };
    std::fs::copy(&lib_path, &unique_name).expect("[ ERROR ] Failed to copy lib to unique path");
    let unique_lib_path = Path::new(&unique_name).canonicalize().unwrap();
    eprintln!("[ INFO ] Loading lib {:?}", unique_lib_path);
    (
        ll::Library::new(unique_lib_path.as_os_str()).unwrap_or_else(|err| {
            panic!(
                "[ ERROR ] Failed to load library '{:?}': {:?}",
                unique_lib_path, err
            )
        }),
        unique_lib_path,
    )
}

#[cfg(debug_assertions)]
pub fn lib_reload(lib_path: &str, unique_path: &mut PathBuf) -> ll::Library {
    if let Err(err) = std::fs::remove_file(&unique_path) {
        eprintln!(
            "[ WARNING ] Failed to remove old lib {:?}: {:?}",
            unique_path, err
        );
    }
    let (lib, path) = lib_load(lib_path);
    *unique_path = path;
    lib
}

pub unsafe fn game_load(game_lib: &ll::Library) -> ll::Result<Game_Api<'_>> {
    Ok(Game_Api {
        init: game_lib.get(b"game_init")?,
        update: game_lib.get(b"game_update")?,
        shutdown: game_lib.get(b"game_shutdown")?,
        unload: game_lib.get(b"game_unload")?,
        reload: game_lib.get(b"game_reload")?,
    })
}
