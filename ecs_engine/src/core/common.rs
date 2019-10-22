pub mod bitset;
pub mod colors;
pub mod direction;
pub mod rand;
pub mod rect;
pub mod serialize;
pub mod stringid;
pub mod transform;
pub mod vector;

pub type Maybe_Error = Result<(), Box<dyn std::error::Error>>;
