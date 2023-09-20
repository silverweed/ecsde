use std::sync::{Arc, Mutex};

pub trait Logger: Send {
    fn log(&mut self, file: &'static str, line: u32, tag: &'static str, msg: &str);

    fn set_log_file_line(&mut self, l: bool);
}

pub type Loggers = Arc<Mutex<Vec<Box<dyn Logger>>>>;

static mut LOGGERS: Option<Loggers> = None;

#[inline]
pub fn emit_log_msg(file: &'static str, line: u32, tag: &'static str, msg: &str) {
    trace!("emit_log_msg");

    if let Some(loggers) = unsafe { LOGGERS.as_mut() } {
        let mut loggers = loggers.lock().unwrap();
        loggers
            .iter_mut()
            .for_each(|logger| logger.log(file, line, tag, msg));
    } else {
        Println_Logger::default().log(file, line, tag, msg);
    }
}

#[derive(Default)]
struct Println_Logger {
    pub log_file_line: bool,
}

impl Logger for Println_Logger {
    fn log(&mut self, file: &'static str, line: u32, tag: &'static str, msg: &str) {
        use std::fmt::Write;

        let mut s = format!("[ {} ] ", tag);
        if self.log_file_line {
            write!(s, "{}:{}  ", file, line).unwrap();
        }
        write!(s, "{}", msg).unwrap();
        if tag == "DEBUG" || tag == "VERBOSE" {
            eprintln!("{}", s);
        } else {
            println!("{}", s);
        }
    }

    fn set_log_file_line(&mut self, l: bool) {
        self.log_file_line = l;
    }
}

/// # Safety
/// This function is not thread-safe
pub unsafe fn create_loggers() -> Loggers {
    let loggers = Arc::new(Mutex::new(vec![]));
    register_loggers(&loggers);
    loggers
}

// This must be separate from create_loggers since we have to call it during hot reload
/// # Safety
/// This function is not thread-safe
#[inline]
pub unsafe fn register_loggers(loggers: &Loggers) {
    LOGGERS = Some(loggers.clone());
}

/// # Safety
/// This function is not thread-safe
#[inline]
pub unsafe fn unregister_loggers() {
    LOGGERS = None;
}

pub fn add_default_logger(loggers: &mut Loggers) {
    let mut loggers = loggers.lock().unwrap();
    loggers.insert(0, Box::new(Println_Logger::default()));
}

pub fn set_log_file_line(loggers: &mut Loggers, idx: usize, log_file_line: bool) {
    let mut loggers = loggers.lock().unwrap();
    loggers.get_mut(idx).map(|l| l.set_log_file_line(log_file_line));
}

pub fn add_logger(loggers: &mut Loggers, logger: Box<dyn Logger>) {
    let mut loggers = loggers.lock().unwrap();
    loggers.push(logger);
}
