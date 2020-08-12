#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

pub mod byte_stream;
pub mod binary_serializable;

pub use byte_stream::Byte_Stream;
pub use binary_serializable::Binary_Serializable;
