pub mod file_watcher;

use libloading as ll;

use {notify::DebouncedEvent, std::path::PathBuf, std::sync::mpsc::SyncSender};

pub struct Game_Dll_File_Watcher {
    file: PathBuf,
    pub reload_pending: SyncSender<()>,
}

impl Game_Dll_File_Watcher {
    pub fn new(file: PathBuf, reload_pending: SyncSender<()>) -> Self {
        Game_Dll_File_Watcher {
            file: file
                .canonicalize()
                .unwrap_or_else(|err| panic!("Failed to canonicalize path {:?}: {}", file, err)),
            reload_pending,
        }
    }
}

impl file_watcher::File_Watcher_Event_Handler for Game_Dll_File_Watcher {
    fn handle(&mut self, event: &DebouncedEvent) {
        match event {
            DebouncedEvent::Write(path)
            | DebouncedEvent::Create(path)
            | DebouncedEvent::Chmod(path)
            | DebouncedEvent::Remove(path) => match path.canonicalize() {
                Ok(canon_path) => {
                    if canon_path == self.file {
                        let _ = self.reload_pending.try_send(());
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

pub fn lib_reload(lib_path: &str, unique_path: &mut PathBuf) -> ll::Library {
    if let Err(err) = std::fs::remove_file(&unique_path) {
        eprintln!(
            "[ WARNING ] Failed to remove old lib {:?}: {:?}",
            unique_path, err
        );
    }
    let super::Lib_Load_Res { lib, path } = super::lib_load(lib_path, true).unwrap();
    *unique_path = path;
    lib
}
