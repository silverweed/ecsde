#[cfg(feature = "tracer")]
use {
    crate::tracer::{Tracers, Tracer},
    std::sync::{Arc, Mutex},
};

#[cfg(feature = "tracer")]
pub type Debug_Tracers = Arc<Mutex<Tracers>>;

#[cfg(feature = "tracer")]
lazy_static! {
    pub static ref DEBUG_TRACERS: Debug_Tracers = Arc::new(Mutex::new(Tracers::new()));
}

#[cfg(feature = "tracer")]
#[macro_export]
macro_rules! trace {
    ($tag: expr) => {
        let _trace_var = $crate::tracer::debug_trace_on_thread($tag, $crate::prelude::DEBUG_TRACERS.clone(), std::thread::current().id());
    };
}

#[cfg(not(feature = "tracer"))]
#[macro_export]
macro_rules! trace {
    ($tag: expr) => {};
}
