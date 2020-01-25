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
#[cfg(not(debug_assertions))]
macro_rules! ldebug {
    ($fmt:tt $(,$arg:expr)* $(,)?) => {
        ()
    };
}
