pub mod base;
pub mod gfx;
pub mod transform;

use std::fmt::Debug;
use typename::TypeName;

pub trait Component: Clone + Default + Debug + TypeName {}
impl<T> Component for T where T: Clone + Default + Debug + TypeName {}
