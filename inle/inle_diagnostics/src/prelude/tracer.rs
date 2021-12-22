use {crate::tracer::Tracers, std::sync::Mutex};

pub type Debug_Tracers = Mutex<Tracers>;

lazy_static! {
    pub static ref DEBUG_TRACERS: Debug_Tracers = Mutex::new(Tracers::new());
}

#[macro_export]
macro_rules! trace {
    ($tag: expr) => {
        let _trace_var = $crate::tracer::debug_trace_on_thread(
            $tag,
            &$crate::prelude::DEBUG_TRACERS,
            std::thread::current().id(),
        );
    };
}
