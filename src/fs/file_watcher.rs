use crate::core::common::Maybe_Error;
use crate::gfx::ui::UI_Request;
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::time::Duration;

pub struct File_Watcher_System {
    watcher: RecommendedWatcher,
    rx: Receiver<DebouncedEvent>,
    ui_req_tx: Option<Sender<UI_Request>>, // Note: for now we keep this "optional" to have a more loosely coupled design (is it, though?). This system is likely gonna change a lot anyway, so this is not final by any measure.
}

impl File_Watcher_System {
    pub fn new() -> File_Watcher_System {
        let (tx, rx) = channel();
        File_Watcher_System {
            watcher: watcher(tx, Duration::from_secs(2)).unwrap(),
            rx,
            ui_req_tx: None,
        }
    }

    // @Incomplete: pass a pattern rather than a path
    pub fn init(&mut self, path: &str, ui_req_tx: Sender<UI_Request>) -> Maybe_Error {
        eprintln!("File_Watcher_System: watching {}", path);
        self.watcher.watch(path, RecursiveMode::Recursive)?;
        self.ui_req_tx = Some(ui_req_tx);
        Ok(())
    }

    pub fn update(&mut self) {
        match self.rx.try_recv() {
            Ok(event) => self.handle_file_event(event),
            Err(err) => match err {
                TryRecvError::Disconnected => println!("Watch error: {:?}", err),
                TryRecvError::Empty => (),
            },
        }
    }

    fn handle_file_event(&mut self, event: DebouncedEvent) {
        // @Incomplete
        let tx = self.ui_req_tx.as_mut().unwrap();
        tx.send(UI_Request::Add_Fadeout_Text(
            format!("{:?}", event),
            Duration::from_secs(2),
        ))
        .unwrap();
    }
}
