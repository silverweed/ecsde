use std::sync::{Arc, Mutex};

pub trait Logger: Send {
    fn log(&mut self, file: &'static str, line: u32, tag: &'static str, msg: &str);
}

lazy_static! {
    static ref LOGGERS: Arc<Mutex<Vec<Box<dyn Logger>>>> = Arc::new(Mutex::new(vec![]));
}

#[inline]
pub fn emit_log_msg(file: &'static str, line: u32, tag: &'static str, msg: &str) {
    trace!("emit_log_msg");

    let mut loggers = LOGGERS.lock().unwrap();
    loggers.iter_mut().for_each(|logger| logger.log(file, line, tag, msg));
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

pub fn add_default_logger() {
    add_logger(Box::new(Println_Logger {}));
}

pub fn add_logger(logger: Box<dyn Logger>) {
    let mut loggers = LOGGERS.lock().unwrap();
    loggers.push(logger);
}
