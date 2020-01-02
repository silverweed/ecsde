use crate::core::common::transform::Transform2D;
use crate::core::common::vector::Vec2f;

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct C_Velocity2D {
    pub x: f32,
    pub y: f32,
}

impl C_Velocity2D {
    pub fn set(&mut self, v: Vec2f) {
        self.x = v.x;
        self.y = v.y;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct C_Spatial2D {
    pub local_transform: Transform2D,
    pub global_transform: Transform2D,
    pub velocity: C_Velocity2D,
}
