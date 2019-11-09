use super::game_api::Game_Api;
use std::path::{Path, PathBuf};

#[cfg(debug_assertions)]
use ecs_engine::fs::file_watcher;
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
        match event {
            DebouncedEvent::Write(path)
            | DebouncedEvent::Chmod(path)
            | DebouncedEvent::Remove(path)
                if *path == self.file =>
            {
                let _ = self.reload_pending.try_send(());
            }
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
    std::fs::remove_file(&unique_path).unwrap_or_else(|err| {
        panic!(
            "[ ERROR ] Failed to remove old lib {:?}: {:?}",
            unique_path, err
        )
    });
    let (lib, path) = lib_load(lib_path);
    *unique_path = path;
    lib
}

pub unsafe fn game_load(game_lib: &ll::Library) -> ll::Result<Game_Api<'_>> {
    Ok(Game_Api {
        init: game_lib.get(b"game_init")?,
        update: game_lib.get(b"game_update")?,
        shutdown: game_lib.get(b"game_shutdown")?,
    })
}
