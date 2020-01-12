use super::vector::Vec2f;

#[derive(Copy, Clone)]
pub struct Circle {
    pub radius: f32,
    pub center: Vec2f,
}

#[derive(Clone)]
pub struct Arrow {
    pub center: Vec2f,
    pub direction: Vec2f, // also includes magnitude
    pub thickness: f32,
    pub arrow_size: f32,
}
