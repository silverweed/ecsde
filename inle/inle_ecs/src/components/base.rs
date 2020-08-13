use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct C_Spatial2D {
    pub transform: Transform2D,
    pub velocity: Vec2f,
    pub frame_starting_pos: Vec2f,
}
