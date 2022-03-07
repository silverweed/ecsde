use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct C_Spatial2D {
    pub transform: Transform2D,
    pub acceleration: Vec2f,
    pub prev_acceleration: Vec2f,
    pub velocity: Vec2f,
    pub frame_starting_pos: Vec2f,
}
