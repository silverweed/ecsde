#![cfg(debug_assertions)]

use inle_common::Maybe_Error;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

pub trait File_Watcher_Event_Handler: Send {
    fn handle(&mut self, evt: &DebouncedEvent);
}

pub struct File_Watch_Config {
    pub recursive_mode: RecursiveMode,
    pub interval: Duration,
}

pub fn start_file_watch(
    path: PathBuf,
    config: File_Watch_Config,
    event_handlers: Vec<Box<dyn File_Watcher_Event_Handler>>,
) -> std::io::Result<thread::JoinHandle<()>> {
    thread::Builder::new()
        .name(format!("fwch_{:?}", path.as_path().file_name().unwrap()))
        .spawn(move || {
            file_watch_listen(path, config, event_handlers).unwrap();
        })
}

fn file_watch_listen(
    path: PathBuf,
    config: File_Watch_Config,
    mut event_handlers: Vec<Box<dyn File_Watcher_Event_Handler>>,
) -> Maybe_Error {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, config.interval).unwrap();
    watcher.watch(path.to_str().unwrap(), config.recursive_mode)?;
    linfo!("Started watching {:?}", path);

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
