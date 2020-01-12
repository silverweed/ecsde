use crate::core::common::vector::Vec2f;

#[derive(Copy, Clone, Debug)]
pub enum Collider_Shape {
    Rect { width: f32, height: f32 },
}

impl Default for Collider_Shape {
    fn default() -> Collider_Shape {
        Collider_Shape::Rect {
            width: 0.,
            height: 0.,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Collider {
    pub shape: Collider_Shape,
    pub offset: Vec2f,
    pub colliding: bool,
}
