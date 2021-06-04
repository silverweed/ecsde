use std::sync::{Arc, Mutex};

pub trait Logger: Send {
    fn log(&mut self, file: &'static str, line: u32, tag: &'static str, msg: &str);
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
        Println_Logger {}.log(file, line, tag, msg);
    }
}

pub struct Println_Logger;

impl Logger for Println_Logger {
    fn log(&mut self, _file: &'static str, _line: u32, tag: &'static str, msg: &str) {
        if tag == "DEBUG" || tag == "VERBOSE" {
            eprintln!("[ {} ] {}", tag, msg);
        } else {
            println!("[ {} ] {}", tag, msg);
        }
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
    add_logger(loggers, Box::new(Println_Logger {}));
}

pub fn add_logger(loggers: &mut Loggers, logger: Box<dyn Logger>) {
    let mut loggers = loggers.lock().unwrap();
    loggers.push(logger);
}
