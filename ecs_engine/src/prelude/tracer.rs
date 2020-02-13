#[cfg(debug_assertions)]
use {
    crate::debug::tracer::Tracer,
    std::sync::{Arc, Mutex},
};

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! trace {
    ($tag: expr, $tracer: expr) => {
        let _trace_var = $crate::debug::tracer::debug_trace($tag, $tracer.clone());
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! trace {
    ($tag: expr, $ng_state: expr) => {};
}

#[cfg(debug_assertions)]
pub type Debug_Tracer = Arc<Mutex<Tracer>>;

#[cfg(not(debug_assertions))]
pub type Debug_Tracer = ();

#[cfg(debug_assertions)]
pub fn new_debug_tracer() -> Debug_Tracer {
    Arc::new(Mutex::new(Tracer::new()))
}

#[cfg(not(debug_assertions))]
pub fn new_debug_tracer() -> Debug_Tracer {
    ()
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! clone_tracer {
    ($tracer: expr) => {
        $tracer.clone()
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! clone_tracer {
    ($tracer: expr) => {
        ()
    };
}
