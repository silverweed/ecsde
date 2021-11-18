use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct Long_Task_Manager {
    executors: Vec<Long_Task_Executor>,
    // Note: we have a single sender/receiver and all executor threads "race" on the same receiver.
    task_sender: Option<Sender<Long_Task_Executor_Msg>>,
}

/// A Long_Task is an async task that spans for potentially more than a frame.
struct Long_Task {
    func: Box<dyn FnOnce() + Send>,
}

impl Long_Task_Manager {
    pub fn start(&mut self, n_threads: usize) {
        assert!(
            self.task_sender.is_none(),
            "Long_Task_Manager started multiple times!"
        );

        let (sender, receiver) = channel();
        self.task_sender = Some(sender);
        let receiver = Arc::new(Mutex::new(receiver));

        self.executors.reserve_exact(n_threads);
        for _ in 0..n_threads {
            self.executors
                .push(Long_Task_Executor::new(receiver.clone()));
        }

        lok!("Started Long_Task_Manager with {} threads.", n_threads);
    }

    pub fn shutdown_blocking(&mut self) {
        linfo!("Starting Long_Task_Manager shutdown...");

        let time_taken = std::time::Instant::now();

        let sender = self
            .task_sender
            .as_mut()
            .expect("Called shutdown_blocking on non-started Long_Task_Manager!");
        for _ in 0..self.executors.len() {
            // Each executor will process Terminate exactly once, so we send one per executor.
            sender.send(Long_Task_Executor_Msg::Terminate);
        }

        for exec in self.executors.drain(..) {
            exec.thread
                .join()
                .expect("Failed to join Long_Task_Executor!");
        }

        linfo!(
            "Long_Task_Manager shutdown completed in {:?}.",
            time_taken.elapsed()
        );
    }

    pub fn create_task<F: 'static + FnOnce() + Send>(&mut self, f: F) {
        let task = Long_Task { func: Box::new(f) };
        self.task_sender
            .as_mut()
            .expect("Called create_task on non-started Long_Task_Manager!")
            .send(Long_Task_Executor_Msg::New_Task(task))
            .unwrap();
    }
}

enum Long_Task_Executor_Msg {
    New_Task(Long_Task),
    Terminate,
}

struct Long_Task_Executor {
    thread: std::thread::JoinHandle<()>,
}

impl Long_Task_Executor {
    fn new(task_recv: Arc<Mutex<Receiver<Long_Task_Executor_Msg>>>) -> Self {
        let thread = std::thread::spawn(move || {
            long_task_executor_loop(task_recv);
        });

        Self { thread }
    }
}

fn long_task_executor_loop(task_recv: Arc<Mutex<Receiver<Long_Task_Executor_Msg>>>) {
    // Currently this function is dead simple: it just receives tasks and processes them synchronously
    // without any kind of time slicing.

    loop {
        match task_recv.lock().unwrap().recv() {
            Ok(msg) => match msg {
                Long_Task_Executor_Msg::Terminate => return,
                Long_Task_Executor_Msg::New_Task(task) => {
                    lverbose!("Processing long task...");
                    (task.func)();
                }
            },
            Err(err) => {
                panic!("Long_Task_Executor panicked with error: {}", err);
            }
        }
    }
}
