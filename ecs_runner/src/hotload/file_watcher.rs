use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

pub trait File_Watcher_Event_Handler: Send {
    fn handle(&mut self, evt: &DebouncedEvent);
}

pub struct File_Watch_Config {
    pub recursive_mode: RecursiveMode,
    pub interval: Duration,
}

pub struct File_Watch_Thread {
    pub handle: thread::JoinHandle<()>,
    pub watcher: RecommendedWatcher,
    pub stop_signal: Sender<()>,
    path: PathBuf,
}

pub fn stop_file_watch(mut fw_thread: File_Watch_Thread) {
    fw_thread.watcher.unwatch(&fw_thread.path).unwrap();
    fw_thread.stop_signal.send(()).unwrap();
    fw_thread.handle.join().unwrap();
}

pub fn start_file_watch(
    path: PathBuf,
    config: File_Watch_Config,
    event_handlers: Vec<Box<dyn File_Watcher_Event_Handler>>,
) -> std::io::Result<File_Watch_Thread> {
    let (wtx, wrx) = channel();
    let (stop_tx, stop_rx) = channel();
    let mut watcher = watcher(wtx, config.interval).unwrap();
    watcher
        .watch(path.to_str().unwrap(), config.recursive_mode)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e)))?;
    eprintln!("Started watching {:?}", path);

    let handle = thread::Builder::new()
        .name(format!(
            "runner_fwch_{:?}",
            path.as_path().file_name().unwrap()
        ))
        .spawn(move || {
            file_watch_listen(wrx, event_handlers, stop_rx).unwrap();
        })?;

    Ok(File_Watch_Thread {
        path,
        handle,
        watcher,
        stop_signal: stop_tx,
    })
}

fn file_watch_listen(
    watcher_rx: Receiver<DebouncedEvent>,
    mut event_handlers: Vec<Box<dyn File_Watcher_Event_Handler>>,
    stop_signal: Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    while stop_signal.try_recv() == Err(TryRecvError::Empty) {
        notify_handlers(&mut event_handlers, watcher_rx.recv()?);
    }

    Ok(())
}

fn notify_handlers(
    event_handlers: &mut [Box<dyn File_Watcher_Event_Handler>],
    event: DebouncedEvent,
) {
    for handler in event_handlers.iter_mut() {
        handler.handle(&event);
    }
}
