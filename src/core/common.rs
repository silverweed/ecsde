pub mod stringid;
pub mod vector;

pub type Maybe_Error = Result<(), Box<dyn std::error::Error>>;
