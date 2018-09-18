use std::fmt::Debug;

pub trait Component: Copy + Clone + Default + Debug {}
impl<T> Component for T where T: Copy + Clone + Default + Debug {}

#[derive(Copy, Clone, Default, Debug)]
pub struct C_Position {
	pub x: f32,
	pub y: f32,
}
