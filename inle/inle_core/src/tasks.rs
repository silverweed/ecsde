use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::Arc;

#[derive(Default)]
pub struct Long_Task_Manager {
    executors: Vec<Long_Task_Executor>,
}

/// A Long_Task is an async task that spans for potentially more than a frame.
struct Long_Task {
    func: Box<dyn FnOnce() + Send>,
}

impl Long_Task_Manager {
    pub fn start(&mut self, n_threads: usize) {
        self.executors.reserve_exact(n_threads);
        for _ in 0..n_threads {
            self.executors.push(Long_Task_Executor::default());
        }

        lok!("Started Long_Task_Manager with {} threads.", n_threads);
    }

    pub fn shutdown_blocking(&mut self) {
        linfo!("Starting Long_Task_Manager shutdown...");

        let time_taken = std::time::Instant::now();

        for exec in &mut self.executors {
            exec.begin_shutdown();
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
        let executor_with_least_tasks = self.executors.iter_mut().min_by_key(|exec| exec.n_tasks_pending())
            .expect("Called create_task on a Long_Task_Manager without executors! This means that it was either never started or already shut down!");
        executor_with_least_tasks.push_task(task);
    }
}

enum Long_Task_Executor_Msg {
    New_Task(Long_Task),
    Terminate,
}

struct Long_Task_Executor {
    thread: std::thread::JoinHandle<()>,
    task_sender: Sender<Long_Task_Executor_Msg>,
    n_tasks_pending: Arc<AtomicU32>,
    waiting_for_termination: bool,
}

impl Default for Long_Task_Executor {
    fn default() -> Self {
        let (task_sender, task_recv) = channel();
        let n_tasks_pending_ro = Arc::new(AtomicU32::new(0));
        let n_tasks_pending_wo = n_tasks_pending_ro.clone();
        let thread = std::thread::spawn(move || {
            long_task_executor_loop(task_recv, n_tasks_pending_wo);
        });

        Self {
            thread,
            task_sender,
            n_tasks_pending: n_tasks_pending_ro,
            waiting_for_termination: false,
        }
    }
}

impl Long_Task_Executor {
    fn n_tasks_pending(&self) -> u32 {
        self.n_tasks_pending.load(Ordering::Acquire)
    }

    fn begin_shutdown(&mut self) {
        self.waiting_for_termination = true;
        self.task_sender
            .send(Long_Task_Executor_Msg::Terminate)
            .unwrap_or_else(|err| lerr!("Failed to send Terminate message to executor: {}", err));
    }

    fn push_task(&mut self, task: Long_Task) {
        lverbose!("Sending task to executor");
        self.task_sender
            .send(Long_Task_Executor_Msg::New_Task(task))
            .expect("Failed to send long task to the executor!");
    }
}

fn long_task_executor_loop(
    task_recv: Receiver<Long_Task_Executor_Msg>,
    n_tasks_pending: Arc<AtomicU32>,
) {
    // Currently this function is dead simple: it just receives tasks and processes them synchronously
    // without any kind of time slicing.
    // If multiple messages are received simultaneously, we prioritize Terminate over all others.

    let mut pending_tasks = vec![];
    loop {
        pending_tasks.clear();
        loop {
            match task_recv.try_recv() {
                Ok(msg) => match msg {
                    Long_Task_Executor_Msg::Terminate => return,
                    Long_Task_Executor_Msg::New_Task(task) => pending_tasks.push(task),
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        if pending_tasks.is_empty() {
            continue;
        }

        lverbose!("Processing {} tasks", pending_tasks.len());
        n_tasks_pending.store(pending_tasks.len() as u32, Ordering::Release);
        for task in pending_tasks.drain(..) {
            (task.func)();
            n_tasks_pending.fetch_sub(1, Ordering::Release);
        }
    }
}
