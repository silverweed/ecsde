#[cfg(debug_assertions)]
use {
    crate::tracer::Tracer,
    std::sync::{Arc, Mutex},
};

#[cfg(debug_assertions)]
pub type Debug_Tracer = Arc<Mutex<Tracer>>;

#[cfg(debug_assertions)]
lazy_static! {
    pub static ref DEBUG_TRACER: Debug_Tracer = Arc::new(Mutex::new(Tracer::new()));
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! trace {
    ($tag: expr) => {
        let _trace_var = $crate::tracer::debug_trace($tag, $crate::prelude::DEBUG_TRACER.clone());
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! trace {
    ($tag: expr) => {};
}
