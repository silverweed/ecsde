use crate::common::angle::{rad, Angle};
use crate::common::matrix::Matrix3;
use crate::common::vector::Vec2f;

#[cfg(feature = "gfx-sfml")]
pub mod sfml;

// Likely @Incomplete: we don't want to recalculate the matrix every time.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform2D {
    position: Vec2f,
    rotation: Angle,
    scale: Vec2f,
}

impl Default for Transform2D {
    fn default() -> Self {
        Transform2D {
            position: Vec2f::new(0.0, 0.0),
            rotation: rad(0.0),
            scale: Vec2f::new(1.0, 1.0),
        }
    }
}

impl Transform2D {
    pub fn new() -> Transform2D {
        Transform2D::default()
    }

    pub fn from_pos_rot_scale(pos: Vec2f, rot: Angle, scale: Vec2f) -> Transform2D {
        let mut t = Transform2D::new();
        t.set_position_v(pos);
        t.set_rotation(rot);
        t.set_scale_v(scale);
        t
    }

    pub fn from_pos(pos: Vec2f) -> Transform2D {
        let mut t = Transform2D::new();
        t.set_position_v(pos);
        t
    }

    pub fn new_from_matrix(m: &Matrix3<f32>) -> Transform2D {
        let sx = (m[0][0] * m[0][0] + m[0][1] * m[0][1]).sqrt();
        let sy = (m[1][0] * m[1][0] + m[1][1] * m[1][1]).sqrt();
        let rot = m[0][1].atan2(m[0][0]);
        let tx = m[2][0];
        let ty = m[2][1];
        Transform2D {
            position: Vec2f::new(tx, ty),
            rotation: rad(rot),
            scale: Vec2f::new(sx, sy),
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

    pub fn set_position_v(&mut self, p: Vec2f) {
        self.position.x = p.x;
        self.position.y = p.y;
    }

    pub fn position(&self) -> Vec2f {
        self.position
    }

    pub fn add_scale(&mut self, x: f32, y: f32) {
        self.scale.x += x;
        self.scale.y += y;
    }

    pub fn add_scale_v(&mut self, s: Vec2f) {
        self.scale.x += s.x;
        self.scale.y += s.y;
    }

    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.scale.x = x;
        self.scale.y = y;
    }

    pub fn set_scale_v(&mut self, s: Vec2f) {
        self.scale.x = s.x;
        self.scale.y = s.y;
    }

    pub fn scale(&self) -> Vec2f {
        self.scale
    }

    pub fn set_rotation(&mut self, angle: Angle) {
        self.rotation = angle;
    }

    pub fn rotate(&mut self, angle: Angle) {
        self.rotation += angle;
    }

    pub fn rotation(&self) -> Angle {
        self.rotation
    }

    pub fn get_matrix(&self) -> Matrix3<f32> {
        let angle = self.rotation.as_rad();
        let angle = -angle;
        let (sine, cosine) = angle.sin_cos();
        let sxc = self.scale.x * cosine;
        let syc = self.scale.y * cosine;
        let sxs = self.scale.x * sine;
        let sys = self.scale.y * sine;
        let tx = self.position.x;
        let ty = self.position.y;

        // R | T
        // 0 | 1
        Matrix3::new(sxc, sys, tx, -sxs, syc, ty, 0.0, 0.0, 1.0)
    }

    pub fn combine(&self, other: &Transform2D) -> Transform2D {
        trace!("transform::combine");
        Transform2D::new_from_matrix(&(self.get_matrix() * other.get_matrix()))
    }

    pub fn inverse(&self) -> Transform2D {
        let s = self.scale();
        Transform2D::from_pos_rot_scale(
            -self.position(),
            -self.rotation(),
            Vec2f::new(1.0 / s.x, 1.0 / s.y),
        )
    }
}

impl std::ops::Mul<Vec2f> for Transform2D {
    type Output = Vec2f;

    fn mul(self, v: Vec2f) -> Self::Output {
        let m = self.get_matrix();
        Vec2f::new(
            m[0][0] * v.x + m[1][0] * v.y + m[2][0],
            m[0][1] * v.x + m[1][1] * v.y + m[2][1],
        )
    }
}

#[cfg(test)]
impl crate::test_common::Approx_Eq_Testable for Transform2D {
    fn cmp_list(&self) -> Vec<f32> {
        vec![
            self.position.x,
            self.position.y,
            self.rotation.as_rad(),
            self.scale.x,
            self.scale.y,
        ]
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

    #[test]
    fn default() {
        let tr = Transform2D::new();
        assert_eq!(tr.position(), Vec2f::new(0.0, 0.0));
        assert_eq!(tr.rotation(), rad(0.0));
        assert_eq!(tr.scale(), Vec2f::new(1.0, 1.0));
    }

    #[test]
    fn translate() {
        let mut tr = Transform2D::new();
        tr.translate(-42.0, 21.0);
        assert_eq!(tr.position(), Vec2f::new(-42.0, 21.0));
        tr.translate(1.5, -21.0);
        assert_eq!(tr.position(), Vec2f::new(-40.5, 0.0));
    }

    #[test]
    fn set_position() {
        let mut tr = Transform2D::new();
        tr.set_position(-222.2, 0.02);
        assert_eq!(tr.position(), Vec2f::new(-222.2, 0.02));
        tr.set_position(11.2, 0.0);
        assert_eq!(tr.position(), Vec2f::new(11.2, 0.0));
    }

    #[test]
    fn add_scale() {
        let mut tr = Transform2D::new();
        tr.add_scale(0.5, -0.5);
        assert_eq!(tr.scale(), Vec2f::new(1.5, 0.5));
        tr.add_scale(1.5, 10.0);
        assert_eq!(tr.scale(), Vec2f::new(3.0, 10.5));
    }

    #[test]
    fn set_scale() {
        let mut tr = Transform2D::new();
        tr.set_scale(0.5, -0.5);
        assert_eq!(tr.scale(), Vec2f::new(0.5, -0.5));
        tr.set_scale(-1.0, 10.5);
        assert_eq!(tr.scale(), Vec2f::new(-1.0, 10.5));
    }

    #[test]
    fn rotate() {
        let mut tr = Transform2D::new();
        tr.rotate(rad(2.0));
        assert_approx_eq!(tr.rotation().as_rad(), 2.0);
        tr.rotate(rad(-1.2));
        assert_approx_eq!(tr.rotation().as_rad(), 0.8);
    }

    #[test]
    fn set_rotation() {
        let mut tr = Transform2D::new();
        tr.set_rotation(rad(2.0));
        assert_approx_eq!(tr.rotation().as_rad(), 2.0);
        tr.set_rotation(rad(-1.2));
        assert_approx_eq!(tr.rotation().as_rad(), -1.2);
    }

    #[test]
    fn to_matrix_from_matrix() {
        let mut t1 = Transform2D::new();
        t1.set_position(100.0, 0.0);
        t1.set_rotation(rad(1.4));
        t1.set_scale(2.0, 2.0);

        let t2 = Transform2D::new_from_matrix(&t1.get_matrix());
        assert_approx_eq!(t2.position().x, 100.0);
        assert_approx_eq!(t2.rotation().as_rad_0tau(), 1.4);
        assert_approx_eq!(t2.scale().y, 2.0);
    }

    #[test]
    fn matrix_from_pos() {
        let t1 = Transform2D::from_pos(v2!(-100., 240.));
        assert_approx_eq!(t1.position(), v2!(-100., 240.));
    }

    #[test]
    fn matrix_from_pos_rot_scale() {
        let t1 = Transform2D::from_pos_rot_scale(v2!(-100., 240.), rad(2.3), v2!(-0.3, 2.3));
        assert_approx_eq!(t1.position(), v2!(-100., 240.));
        assert_approx_eq!(t1.rotation(), rad(2.3));
        assert_approx_eq!(t1.scale(), v2!(-0.3, 2.3));
    }

    #[test]
    fn matrix_combine() {
        let t1 = Transform2D::from_pos(v2!(2., 45.));
        let t2 = Transform2D::default().combine(&t1);
        assert_approx_eq!(t2.position(), v2!(2., 45.));

        let t3 = t2.inverse().combine(&t2);
        assert_approx_eq!(t3, Transform2D::default());

        let t4 = Transform2D::from_pos_rot_scale(v2!(0., 30.), rad(1.), v2!(1., 1.2));
        let mut t5 = t4.combine(&t3);
        assert_approx_eq!(t5, t4);
        assert_approx_eq!(t5.position(), v2!(0., 30.));

        t5.translate(20., -100.);
        assert_approx_eq!(t5.position(), v2!(20., -70.));
    }
}
