#[macro_use]
mod log;

#[cfg(feature = "tracer")]
#[macro_use]
mod tracer;

pub use log::*;

#[cfg(feature = "tracer")]
pub use tracer::*;

#[cfg(not(feature = "tracer"))]
#[macro_export]
macro_rules! trace {
    ($tag: expr) => {};
}
