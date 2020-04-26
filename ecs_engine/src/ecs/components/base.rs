use crate::common::transform::Transform2D;
use crate::common::vector::Vec2f;

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct C_Spatial2D {
    pub transform: Transform2D,
    pub velocity: Vec2f,
}
