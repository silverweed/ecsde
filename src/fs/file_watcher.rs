use crate::core::common::Maybe_Error;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

pub trait File_Watcher_Event_Handler: Sync + Send {
    fn handle(&mut self, evt: &DebouncedEvent);
}

pub fn start_file_watch(
    path: PathBuf,
    event_handlers: Vec<Box<dyn File_Watcher_Event_Handler>>,
) -> Result<thread::JoinHandle<()>, std::io::Error> {
    thread::Builder::new()
        .name(format!("fwch_{:?}", path.as_path().file_name().unwrap()))
        .spawn(move || {
            file_watch_listen(path, event_handlers).unwrap();
        })
}

fn file_watch_listen(
    path: PathBuf,
    mut event_handlers: Vec<Box<dyn File_Watcher_Event_Handler>>,
) -> Maybe_Error {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
    watcher.watch(path.to_str().unwrap(), RecursiveMode::Recursive)?;
    eprintln!("Started watching {:?}", path);

    loop {
        notify_handlers(&mut event_handlers, rx.recv()?);
    }
}

fn notify_handlers(
    event_handlers: &mut [Box<dyn File_Watcher_Event_Handler>],
    event: DebouncedEvent,
) {
    for handler in event_handlers.iter_mut() {
        handler.handle(&event);
    }
}
