use crate::core::common::vector::Vec2f;
use cgmath::{Matrix3, Rad};
use std::convert::Into;
use typename::TypeName;

// Likely @Incomplete: we don't want to recalculate the matrix every time.
#[derive(Copy, Clone, Debug, TypeName, PartialEq)]
pub struct C_Transform2D {
    position: Vec2f,
    rotation: Rad<f32>,
    scale: Vec2f,
    origin: Vec2f,
}

impl Default for C_Transform2D {
    fn default() -> Self {
        C_Transform2D {
            position: Vec2f::new(0.0, 0.0),
            rotation: Rad(0.0),
            scale: Vec2f::new(1.0, 1.0),
            origin: Vec2f::new(0.0, 0.0),
        }
    }
}

impl C_Transform2D {
    pub fn new() -> C_Transform2D {
        C_Transform2D::default()
    }

    pub fn new_from_matrix(m: &Matrix3<f32>) -> C_Transform2D {
        let sx = (m[0][0] * m[0][0] + m[0][1] * m[0][1]).sqrt();
        let sy = (m[1][0] * m[1][0] + m[1][1] * m[1][1]).sqrt();
        let rot = m[0][1].atan2(m[0][0]);
        let tx = m[2][0];
        let ty = m[2][1];
        C_Transform2D {
            position: Vec2f::new(tx, ty),
            rotation: Rad(rot),
            scale: Vec2f::new(sx, sy),
            origin: Vec2f::new(0.0, 0.0), // @Incomplete
        }
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.position.x += x;
        self.position.y += y;
    }

    pub fn translate_v(&mut self, v: Vec2f) {
        self.position.x += v.x;
        self.position.y += v.y;
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

    pub fn set_origin(&mut self, x: f32, y: f32) {
        self.origin = Vec2f::new(x, y);
    }

    pub fn get_matrix(&self) -> Matrix3<f32> {
        let Rad(angle) = self.rotation;
        let angle = -angle;
        let cosine = angle.cos();
        let sine = angle.sin();
        let sxc = self.scale.x * cosine;
        let syc = self.scale.y * cosine;
        let sxs = self.scale.x * sine;
        let sys = self.scale.y * sine;
        let tx = -self.origin.x * sxc - self.origin.y * sys + self.position.x;
        let ty = self.origin.x * sxs - self.origin.y * syc + self.position.y;

        // R | T
        // 0 | 1
        Matrix3::new(sxc, -sys, 0.0, sxs, syc, 0.0, tx, ty, 1.0)
    }

    #[cfg(feature = "use-sfml")]
    pub fn get_matrix_sfml(&self) -> sfml::graphics::Transform {
        let Rad(angle) = self.rotation;
        let angle = -angle;
        let cosine = angle.cos();
        let sine = angle.sin();
        let sxc = self.scale.x * cosine;
        let syc = self.scale.y * cosine;
        let sxs = self.scale.x * sine;
        let sys = self.scale.y * sine;
        let tx = -self.origin.x * sxc - self.origin.y * sys + self.position.x;
        let ty = self.origin.x * sxs - self.origin.y * syc + self.position.y;

        // R | 0
        // T | 1
        sfml::graphics::Transform::new(sxc, sys, tx, -sxs, syc, ty, 0.0, 0.0, 1.0)
    }
}

// Note: Matrix3 is column-major
pub fn matrix_pretty_print(m: &Matrix3<f32>) {
    println!(
        "
{:.2} {:.2} {:.2}
{:.2} {:.2} {:.2}
{:.2} {:.2} {:.2}
",
        m[0][0], m[1][0], m[2][0], m[0][1], m[1][1], m[2][1], m[0][2], m[1][2], m[2][2]
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::ApproxEq;

    #[test]
    fn default() {
        let tr = C_Transform2D::new();
        assert_eq!(tr.position(), Vec2f::new(0.0, 0.0));
        assert_eq!(tr.rotation(), Rad(0.0));
        assert_eq!(tr.scale(), Vec2f::new(1.0, 1.0));
    }

    #[test]
    fn translate() {
        let mut tr = C_Transform2D::new();
        tr.translate(-42.0, 21.0);
        assert_eq!(tr.position(), Vec2f::new(-42.0, 21.0));
        tr.translate(1.5, -21.0);
        assert_eq!(tr.position(), Vec2f::new(-40.5, 0.0));
    }

    #[test]
    fn set_position() {
        let mut tr = C_Transform2D::new();
        tr.set_position(-222.2, 0.02);
        assert_eq!(tr.position(), Vec2f::new(-222.2, 0.02));
        tr.set_position(11.2, 0.0);
        assert_eq!(tr.position(), Vec2f::new(11.2, 0.0));
    }

    #[test]
    fn add_scale() {
        let mut tr = C_Transform2D::new();
        tr.add_scale(0.5, -0.5);
        assert_eq!(tr.scale(), Vec2f::new(1.5, 0.5));
        tr.add_scale(1.5, 10.0);
        assert_eq!(tr.scale(), Vec2f::new(3.0, 10.5));
    }

    #[test]
    fn set_scale() {
        let mut tr = C_Transform2D::new();
        tr.set_scale(0.5, -0.5);
        assert_eq!(tr.scale(), Vec2f::new(0.5, -0.5));
        tr.set_scale(-1.0, 10.5);
        assert_eq!(tr.scale(), Vec2f::new(-1.0, 10.5));
    }

    #[test]
    fn rotate() {
        let mut tr = C_Transform2D::new();
        tr.rotate(Rad(2.0));
        assert_approx_eq(tr.rotation(), Rad(2.0));
        tr.rotate(Rad(-1.2));
        assert_approx_eq(tr.rotation(), Rad(0.8));
    }

    #[test]
    fn set_rotation() {
        let mut tr = C_Transform2D::new();
        tr.set_rotation(Rad(2.0));
        assert_approx_eq(tr.rotation(), Rad(2.0));
        tr.set_rotation(Rad(-1.2));
        assert_approx_eq(tr.rotation(), Rad(-1.2));
    }

    #[test]
    fn to_matrix_from_matrix() {
        let mut t1 = C_Transform2D::new();
        let mut t2 = C_Transform2D::new();

        t1.set_position(100.0, 0.0);
        t1.set_rotation(Rad(1.4));
        t1.set_scale(2.0, 2.0);
        t2 = C_Transform2D::new_from_matrix(&t1.get_matrix());
        assert!(t2.position().x.approx_eq(100.0, (0.0, 2)));
        assert_approx_eq(t2.rotation(), Rad(1.4));
        assert!(t2.scale().y.approx_eq(2.0, (0.0, 2)));
    }

    fn assert_approx_eq(a: Rad<f32>, b: Rad<f32>) {
        let Rad(a) = a;
        let Rad(b) = b;
        assert!(a.approx_eq(b, (0.0, 2)), "Expected: {}, Got: {}", b, a);
    }
}
