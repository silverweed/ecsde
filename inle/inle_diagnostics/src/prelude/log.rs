use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicBool, Ordering};

lazy_static! {
    pub static ref ONCE_LOGS: Arc<Mutex<HashSet<String>>> =
        Arc::new(Mutex::new(HashSet::default()));
}

static mut VERBOSE: AtomicBool = AtomicBool::new(false);

#[inline(always)]
pub fn is_verbose() -> bool {
    #[cfg(debug_assertions)]
    unsafe {
        VERBOSE.load(Ordering::Acquire)
    }
    #[cfg(not(debug_assertions))]
    {
        false
    }
}

#[inline(always)]
pub fn set_verbose(verbose: bool) {
    #[cfg(debug_assertions)]
    unsafe {
        VERBOSE.store(verbose, Ordering::Release);
    }
}

#[macro_export]
macro_rules! fatal {
    ($fmt:tt $(,$arg:expr)* $(,)?) => {
        panic!("[ FATAL ] {}", format_args!($fmt, $($arg),*));
    };
}

#[macro_export]
macro_rules! log {
    ($prelude:tt, $($arg:expr),* $(,)*) => {
        println!("[ {} ] {}", $prelude, $($arg),*);
    };
}

#[macro_export]
macro_rules! elog {
    ($prelude:tt, $($arg:expr),* $(,)*) => {
        eprintln!("[ {} ] {}", $prelude, $($arg),*);
    };
}

#[macro_export]
macro_rules! lok {
    ($fmt:tt $(,$arg:expr)* $(,)?) => {
        log!("OK", format_args!($fmt, $($arg),*));
    };
}

#[macro_export]
macro_rules! lerr {
    ($fmt:tt $(,$arg:expr)* $(,)?) => {
        log!("ERROR", format_args!($fmt, $($arg),*));
    };
}

#[macro_export]
macro_rules! lwarn {
    ($fmt:tt $(,$arg:expr)* $(,)?) => {
        log!("WARNING", format_args!($fmt, $($arg),*));
    };
}

#[macro_export]
macro_rules! linfo {
    ($fmt:tt $(,$arg:expr)* $(,)?) => {
        log!("INFO", format_args!($fmt, $($arg),*));
    };
}

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! ldebug {
    ($fmt:tt $(,$arg:expr)* $(,)?) => {
        elog!("DEBUG", format_args!($fmt, $($arg),*));
    };
}

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! lverbose {
    ($fmt:tt $(,$arg:expr)* $(,)?) => {
        if $crate::prelude::is_verbose() {
            elog!("VERBOSE", format_args!($fmt, $($arg),*));
        }
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! ldebug {
    ($fmt:tt $(,$arg:expr)* $(,)?) => {
        ()
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! lverbose {
    ($fmt:tt $(,$arg:expr)* $(,)?) => {
        ()
    };
}

#[macro_export]
macro_rules! log_once {
    ($key: expr, $prelude: tt, $($arg: expr),* $(,)*) => {
        unsafe {
            let mut logs = $crate::prelude::ONCE_LOGS.lock().unwrap();
            if logs.contains($key) {
            } else {
                println!("[ {} ] {}", $prelude, $($arg),*);
                logs.insert(String::from($key));
            }
        }
    }
}

#[macro_export]
macro_rules! elog_once {
    ($key: expr, $prelude: tt, $($arg: expr),* $(,)*) => {
        unsafe {
            let mut logs = $crate::prelude::ONCE_LOGS.lock().unwrap();
            if logs.contains($key) {
            } else {
                elog!($prelude, $($arg),*);
                logs.insert(String::from($key));
            }
        }
    }
}

#[macro_export]
macro_rules! lok_once {
    ($key:expr, $fmt:tt $(,$arg:expr)* $(,)?) => {
        log_once!($key, "OK", format_args!($fmt, $($arg),*));
    };
}

#[macro_export]
macro_rules! lerr_once {
    ($key:expr, $fmt:tt $(,$arg:expr)* $(,)?) => {
        log_once!($key, "ERROR", format_args!($fmt, $($arg),*));
    };
}

#[macro_export]
macro_rules! lwarn_once {
    ($key:expr, $fmt:tt $(,$arg:expr)* $(,)?) => {
        log_once!($key, "WARNING", format_args!($fmt, $($arg),*));
    };
}

#[macro_export]
macro_rules! linfo_once {
    ($key:expr, $fmt:tt $(,$arg:expr)* $(,)?) => {
        log_once!($key, "INFO", format_args!($fmt, $($arg),*));
    };
}

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! ldebug_once {
    ($key:expr, $fmt:tt $(,$arg:expr)* $(,)?) => {
        elog_once!($key, "DEBUG", format_args!($fmt, $($arg),*));
    };
}
