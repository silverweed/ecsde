use crate::core::common::Maybe_Error;
use crate::gfx::ui::UI_Request;
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

pub fn file_watcher_create(path: PathBuf, ui_req_tx: Sender<UI_Request>) -> Maybe_Error {
    Ok(thread::Builder::new()
        .name(format!("file_watcher_{:?}", path))
        .spawn(move || {
            file_watch_listen(path, ui_req_tx).unwrap();
        })
        .map(|_| ())?)
}

fn file_watch_listen(path: PathBuf, mut ui_req_tx: Sender<UI_Request>) -> Maybe_Error {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();
    watcher.watch(path.to_str().unwrap(), RecursiveMode::Recursive)?;
    eprintln!("Started watching {:?}", path);

    loop {
        handle_file_event(&mut ui_req_tx, rx.recv()?);
    }
}

fn handle_file_event(ui_req_tx: &mut Sender<UI_Request>, event: DebouncedEvent) {
    // @Incomplete
    eprintln!("Event");
    ui_req_tx
        .send(UI_Request::Add_Fadeout_Text(
            format!("{:?}", event),
            Duration::from_secs(2),
        ))
        .unwrap();
}
