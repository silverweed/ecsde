use crate::core::common::transform::Transform2D;
use crate::core::common::vector::Vec2f;

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct C_Spatial2D {
    pub local_transform: Transform2D,
    pub global_transform: Transform2D,
    pub velocity: Vec2f,
}
