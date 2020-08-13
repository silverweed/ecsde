#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

pub mod binary_serializable;
pub mod byte_stream;

pub use binary_serializable::Binary_Serializable;
pub use byte_stream::Byte_Stream;
