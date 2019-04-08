pub mod direction;
pub mod stringid;
pub mod transform;
pub mod vector;

pub type Maybe_Error = Result<(), Box<dyn std::error::Error>>;
