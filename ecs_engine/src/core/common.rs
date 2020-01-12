pub mod bitset;
pub mod colors;
pub mod direction;
pub mod math;
pub mod rect;
pub mod serialize;
pub mod shapes;
pub mod stringid;
pub mod transform;
pub mod vector;

pub type Maybe_Error = Result<(), Box<dyn std::error::Error>>;
