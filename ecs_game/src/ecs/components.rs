pub mod base;
pub mod gfx;

use std::fmt::Debug;

pub trait Component: Clone + Default + Debug {}
impl<T> Component for T where T: Clone + Default + Debug {}
