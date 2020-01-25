#[cfg(debug_assertions)]
#[macro_export]
macro_rules! trace {
    ($tag: expr, $tracer: expr) => {
        let _trace_var = crate::debug::tracer::debug_trace($tag, $tracer.clone());
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! trace {
    ($tag: expr, $ng_state: expr) => {};
}

#[cfg(debug_assertions)]
pub type Debug_Tracer = std::rc::Rc<std::cell::RefCell<crate::debug::tracer::Tracer>>;

#[cfg(not(debug_assertions))]
pub type Debug_Tracer = ();

#[cfg(debug_assertions)]
pub fn new_debug_tracer() -> Debug_Tracer {
    std::rc::Rc::new(std::cell::RefCell::new(crate::debug::tracer::Tracer::new()))
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
