pub mod align;
pub mod colors;
pub mod direction;
pub mod rect;
pub mod serialize;
pub mod stringid;
pub mod vector;

pub type Maybe_Error = Result<(), Box<dyn std::error::Error>>;
