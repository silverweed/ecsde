use super::transform::C_Transform2D;
use crate::core::common::vector::Vec2f;
use typename::TypeName;

#[derive(Copy, Clone, Debug, TypeName, PartialEq, Default)]
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

#[derive(Copy, Clone, Debug, TypeName, PartialEq, Default)]
pub struct C_Spatial2D {
    pub transform: C_Transform2D,
    pub velocity: C_Velocity2D,
}
