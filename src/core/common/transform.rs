use crate::core::common::vector::Vec2f;
use cgmath::{Angle, Matrix3, Rad, SquareMatrix};
use std::convert::Into;
use std::ops::Add;
use typename::TypeName;

#[derive(Copy, Clone, Debug, TypeName, PartialEq)]
pub struct C_Transform2D {
    position: Vec2f,
    rotation: Rad<f32>,
    scale: Vec2f,
}

impl Default for C_Transform2D {
    fn default() -> Self {
        C_Transform2D {
            position: Vec2f::new(0.0, 0.0),
            rotation: Rad(0.0),
            scale: Vec2f::new(1.0, 1.0),
        }
    }
}

impl C_Transform2D {
    pub fn new() -> C_Transform2D {
        C_Transform2D::default()
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.position.x += x;
        self.position.y += y;
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position.x = x;
        self.position.y = y;
    }

    pub fn position(&self) -> Vec2f {
        self.position
    }

    pub fn add_scale(&mut self, x: f32, y: f32) {
        self.scale.x += x;
        self.scale.y += y;
    }

    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.scale.x = x;
        self.scale.y = y;
    }

    pub fn scale(&self) -> Vec2f {
        self.scale
    }

    pub fn set_rotation<T: Into<Rad<f32>>>(&mut self, angle: T) {
        self.rotation = angle.into();
    }

    pub fn rotate<T: Into<Rad<f32>>>(&mut self, angle: T) {
        self.rotation += angle.into();
    }

    pub fn rotation(&self) -> Rad<f32> {
        self.rotation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let tr = C_Transform2D::new();
        assert_eq!(tr.position(), Vec2f::new(0.0, 0.0));
        assert_eq!(tr.rotation(), Rad(0.0));
        assert_eq!(tr.scale(), Vec2f::new(1.0, 1.0));
    }

    #[test]
    fn test_translate() {
        let mut tr = C_Transform2D::new();
        tr.translate(-42.0, 21.0);
        assert_eq!(tr.position(), Vec2f::new(-42.0, 21.0));
        tr.translate(1.5, -21.0);
        assert_eq!(tr.position(), Vec2f::new(-40.5, 0.0));
    }

    #[test]
    fn test_set_position() {
        let mut tr = C_Transform2D::new();
        tr.set_position(-222.2, 0.02);
        assert_eq!(tr.position(), Vec2f::new(-222.2, 0.02));
        tr.set_position(11.2, 0.0);
        assert_eq!(tr.position(), Vec2f::new(11.2, 0.0));
    }

    #[test]
    fn test_add_scale() {
        let mut tr = C_Transform2D::new();
        tr.add_scale(0.5, -0.5);
        assert_eq!(tr.scale(), Vec2f::new(1.5, 0.5));
        tr.add_scale(1.5, 10.0);
        assert_eq!(tr.scale(), Vec2f::new(3.0, 10.5));
    }

    #[test]
    fn test_set_scale() {
        let mut tr = C_Transform2D::new();
        tr.set_scale(0.5, -0.5);
        assert_eq!(tr.scale(), Vec2f::new(0.5, -0.5));
        tr.set_scale(-1.0, 10.5);
        assert_eq!(tr.scale(), Vec2f::new(-1.0, 10.5));
    }

    #[test]
    fn test_rotate() {
        let mut tr = C_Transform2D::new();
        tr.rotate(Rad(2.0));
        assert_approx_eq(tr.rotation(), Rad(2.0));
        tr.rotate(Rad(-1.2));
        assert_approx_eq(tr.rotation(), Rad(0.8));
    }

    #[test]
    fn test_set_rotation() {
        let mut tr = C_Transform2D::new();
        tr.set_rotation(Rad(2.0));
        assert_approx_eq(tr.rotation(), Rad(2.0));
        tr.set_rotation(Rad(-1.2));
        assert_approx_eq(tr.rotation(), Rad(-1.2));
    }

    fn assert_approx_eq(a: Rad<f32>, b: Rad<f32>) {
        let Rad(a) = a;
        let Rad(b) = b;
        if (a - b).abs() > 1e-6 {
            assert!(false, "Expected: {}, Got: {}", b, a);
        }
    }
}
