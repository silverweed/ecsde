use crate::common::vector::Vec2f;

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum Collision_Shape {
    Rect { width: f32, height: f32 },
    Circle { radius: f32 },
}

impl Default for Collision_Shape {
    fn default() -> Collision_Shape {
        Collision_Shape::Circle { radius: 0. }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Collider {
    pub shape: Collision_Shape,
    pub offset: Vec2f,
    pub colliding: bool,
}
