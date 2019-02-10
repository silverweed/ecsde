use std::fmt::Debug;
use typename::TypeName;

pub trait Component: Copy + Clone + Default + Debug + TypeName {}
impl<T> Component for T where T: Copy + Clone + Default + Debug + TypeName {}

#[derive(Copy, Clone, Default, Debug, TypeName)] // @Convenience: there's gotta be a better way to say this is a Component
pub struct C_Position {
	pub x: f32,
	pub y: f32,
}
