pub mod angle;
pub mod bitset;
pub mod colors;
pub mod math;
pub mod matrix;
pub mod rect;
pub mod serialize;
pub mod shapes;
pub mod stringid;
pub mod transform;
pub mod units;
pub mod vector;

pub type Maybe_Error = Result<(), Box<dyn std::error::Error>>;
pub const WORD_SIZE: usize = std::mem::size_of::<usize>();
