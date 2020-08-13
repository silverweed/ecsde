#[cfg(feature = "tracer")]
use {
    crate::tracer::Tracer,
    std::sync::{Arc, Mutex},
};

#[cfg(feature = "tracer")]
pub type Debug_Tracer = Arc<Mutex<Tracer>>;

#[cfg(feature = "tracer")]
lazy_static! {
    pub static ref DEBUG_TRACER: Debug_Tracer = Arc::new(Mutex::new(Tracer::new()));
}

#[cfg(feature = "tracer")]
#[macro_export]
macro_rules! trace {
    ($tag: expr) => {
        let _trace_var = $crate::tracer::debug_trace($tag, $crate::prelude::DEBUG_TRACER.clone());
    };
}

#[cfg(not(feature = "tracer"))]
#[macro_export]
macro_rules! trace {
    ($tag: expr) => {};
}
