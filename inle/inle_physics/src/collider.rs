use super::layers::Collision_Layer;
use super::phys_world::{Collider_Handle, Physics_Body_Handle};
use inle_math::vector::Vec2f;

#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Collision_Shape {
    Rect { width: f32, height: f32 },
    Circle { radius: f32 },
}

impl Collision_Shape {
    pub fn extent(self) -> Vec2f {
        match self {
            Collision_Shape::Circle { radius } => v2!(radius, radius) * 2.,
            Collision_Shape::Rect { width, height } => v2!(width, height),
        }
    }
}

impl Default for Collision_Shape {
    fn default() -> Collision_Shape {
        Collision_Shape::Circle { radius: 0. }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Collider {
    pub shape: Collision_Shape,
    pub offset: Vec2f,
    pub is_static: bool,
    pub layer: Collision_Layer,

    // This is written by the Physics_World when the collider is added
    pub handle: Collider_Handle,

    // These should not be written except by the physics system.
    pub position: Vec2f,
    pub velocity: Vec2f,
}
