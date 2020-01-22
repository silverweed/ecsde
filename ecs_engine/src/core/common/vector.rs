use super::math;
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

pub type Vec2u = Vector2<u32>;
pub type Vec2f = Vector2<f32>;
pub type Vec2i = Vector2<i32>;

#[cfg(feature = "use-sfml")]
impl From<Vec2f> for sfml::system::Vector2f {
    fn from(v: Vec2f) -> sfml::system::Vector2f {
        sfml::system::Vector2f::new(v.x, v.y)
    }
}

#[cfg(feature = "use-sfml")]
impl From<sfml::system::Vector2f> for Vec2f {
    fn from(v: sfml::system::Vector2f) -> Vec2f {
        Vec2f::new(v.x, v.y)
    }
}

impl<T> Vector2<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> Vector2<T>
where
    T: Copy
        + Add<Output = T>
        + Mul<Output = T>
        + std::convert::Into<f32>
        + std::convert::From<f32>
        + Default,
{
    pub fn magnitude2(self) -> T {
        self.x * self.x + self.y * self.y
    }

    pub fn magnitude(self) -> f32 {
        self.magnitude2().into().sqrt()
    }

    pub fn normalized_or_zero(self) -> Vector2<T> {
        let mag = self.magnitude2().into();
        if mag == 0. {
            return Self::default();
        }

        let den = math::fast_invsqrt(mag);
        Self {
            x: T::from(self.x.into() * den),
            y: T::from(self.y.into() * den),
        }
    }
}

impl<T: Default> Default for Vector2<T> {
    fn default() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
        }
    }
}

impl<T: Copy> Copy for Vector2<T> {}

impl<T: Clone> Clone for Vector2<T> {
    fn clone(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: self.y.clone(),
        }
    }
}

impl<T: Debug> Debug for Vector2<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{{ x: {:?}, y: {:?} }}", self.x, self.y)
    }
}

impl<T: PartialEq> PartialEq for Vector2<T> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl<T: Eq> Eq for Vector2<T> {}

impl<T: Copy + Neg<Output = T>> Neg for Vector2<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T: Copy + Add<Output = T>> Add for Vector2<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: Copy + Sub<Output = T>> Sub for Vector2<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Copy + Mul<Output = T>> Mul<T> for Vector2<T> {
    type Output = Self;

    fn mul(self, other: T) -> Self::Output {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl<T: Copy + Div<Output = T>> Div<T> for Vector2<T> {
    type Output = Self;

    fn div(self, other: T) -> Self::Output {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl<T: Copy + Add<Output = T>> AddAssign for Vector2<T> {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

impl<T: Copy + Sub<Output = T>> SubAssign for Vector2<T> {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
        };
    }
}

impl<T: Copy + Mul<Output = T>> MulAssign<T> for Vector2<T> {
    fn mul_assign(&mut self, other: T) {
        *self = Self {
            x: self.x * other,
            y: self.y * other,
        };
    }
}

impl<T: Copy + Div<Output = T>> DivAssign<T> for Vector2<T> {
    fn div_assign(&mut self, other: T) {
        *self = Self {
            x: self.x / other,
            y: self.y / other,
        };
    }
}

impl Vec2f {
    // @WaitForStable: make this const as soon as sqrt() is stable as const
    pub fn distance(&self, other: &Self) -> f32 {
        self.distance2(other).sqrt()
    }

    // @WaitForStable: make this const as soon as `-` on f32 is stable as const
    pub fn distance2(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

impl Vec2i {
    pub fn distance(&self, other: &Self) -> f32 {
        self.distance2(other).sqrt()
    }

    pub const fn distance2(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy) as f32
    }
}

impl Vec2u {
    pub fn distance(&self, other: &Self) -> f32 {
        self.distance2(other).sqrt()
    }

    pub const fn distance2(&self, other: &Self) -> f32 {
        let dx = self.x as i32 - other.x as i32;
        let dy = self.y as i32 - other.y as i32;
        (dx * dx + dy * dy) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common::assert_approx_eq;

    #[test]
    fn vector_default() {
        assert_eq!(Vec2u::default(), Vec2u::new(0, 0));
        assert_eq!(Vec2i::default(), Vec2i::new(0, 0));
        assert_eq!(Vec2f::default(), Vec2f::new(0., 0.));
    }

    #[test]
    fn vector_copy() {
        let a = Vec2i::new(2, 3);
        let b = a;
        assert_eq!(a, b);

        let a = Vec2u::new(2, 3);
        let b = a;
        assert_eq!(a, b);

        let a = Vec2f::new(2., 3.);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn vector_add() {
        assert_eq!(Vec2i::new(0, 1) + Vec2i::new(3, 1), Vec2i::new(3, 2));
        assert_eq!(
            Vec2u::new(2, 10) + Vec2u::new(103, 100),
            Vec2u::new(105, 110)
        );
        assert_eq!(Vec2f::new(5., 0.) + Vec2f::new(-2., 9.), Vec2f::new(3., 9.));
    }

    #[test]
    fn vector_sub() {
        assert_eq!(Vec2i::new(0, 1) - Vec2i::new(3, 1), Vec2i::new(-3, 0));
        assert_eq!(Vec2u::new(200, 10) - Vec2u::new(103, 10), Vec2u::new(97, 0));
        assert_eq!(
            Vec2f::new(5., 0.) - Vec2f::new(-2., 9.),
            Vec2f::new(7., -9.)
        );
    }

    #[test]
    fn vector_mul() {
        assert_eq!(Vec2i::new(0, 3) * 3, Vec2i::new(0, 9));
        assert_eq!(Vec2u::new(200, 10) * 2, Vec2u::new(400, 20));
        assert_eq!(Vec2f::new(5., 0.) * 0.5, Vec2f::new(2.5, 0.));
    }

    #[test]
    fn vector_div() {
        assert_eq!(Vec2i::new(0, 3) / 3, Vec2i::new(0, 1));
        assert_eq!(Vec2u::new(200, 10) / 2, Vec2u::new(100, 5));
        assert_eq!(Vec2f::new(5., 0.) / 0.5, Vec2f::new(10., 0.));
    }

    #[test]
    fn vector_neg() {
        assert_eq!(-Vec2i::new(10, 3), Vec2i::new(-10, -3));
        assert_eq!(-Vec2f::new(5., 0.5), Vec2f::new(-5., -0.5));
    }

    #[test]
    fn vector_add_assign() {
        let mut a = Vec2i::new(1, 2);
        a += Vec2i::new(3, 0);
        assert_eq!(a, Vec2i::new(4, 2));

        let mut a = Vec2u::new(1, 2);
        a += Vec2u::new(3, 0);
        assert_eq!(a, Vec2u::new(4, 2));

        let mut a = Vec2f::new(1., 2.);
        a += Vec2f::new(3., 0.);
        assert_eq!(a, Vec2f::new(4., 2.));
    }

    #[test]
    fn vector_sub_assign() {
        let mut a = Vec2i::new(1, 2);
        a -= Vec2i::new(3, 0);
        assert_eq!(a, Vec2i::new(-2, 2));

        let mut a = Vec2u::new(5, 2);
        a -= Vec2u::new(3, 0);
        assert_eq!(a, Vec2u::new(2, 2));

        let mut a = Vec2f::new(1., 2.);
        a -= Vec2f::new(3., 0.);
        assert_eq!(a, Vec2f::new(-2., 2.));
    }

    #[test]
    fn vector_mul_assign() {
        let mut a = Vec2i::new(1, 2);
        a *= 3;
        assert_eq!(a, Vec2i::new(3, 6));

        let mut a = Vec2u::new(1, 2);
        a *= 10;
        assert_eq!(a, Vec2u::new(10, 20));

        let mut a = Vec2f::new(1., 2.);
        a *= 0.5;
        assert_eq!(a, Vec2f::new(0.5, 1.));
    }

    #[test]
    fn vector_div_assign() {
        let mut a = Vec2i::new(4, 8);
        a /= 3;
        assert_eq!(a, Vec2i::new(1, 2));

        let mut a = Vec2u::new(100, 62);
        a /= 10;
        assert_eq!(a, Vec2u::new(10, 6));

        let mut a = Vec2f::new(1., 2.);
        a /= 0.5;
        assert_eq!(a, Vec2f::new(2., 4.));
    }

    #[test]
    fn vector_normalize() {
        let v = Vec2f::new(1., 1.).normalized_or_zero();
        assert_approx_eq(v.x, 0.707_106_7);
        assert_eq!(v.x, v.y);

        assert_eq!(Vec2f::new(0., 0.).normalized_or_zero(), Vec2f::new(0., 0.));
    }

    #[cfg(feature = "use-sfml")]
    #[test]
    fn to_from_framework() {
        let a = Vec2f::new(3., 2.);
        let b: sfml::system::Vector2f = a.into();
        assert_eq!(a.x, b.x);
        assert_eq!(a.y, b.y);

        let c: Vec2f = b.into();
        assert_eq!(c.x, b.x);
        assert_eq!(c.y, b.y);
    }

    #[test]
    fn vector_distance_f32() {
        let a = Vec2f::new(0., 0.);
        let b = Vec2f::new(3., 4.);

        assert_approx_eq(a.distance(&b), 5.);
        assert_approx_eq(b.distance(&a), 5.);

        assert_approx_eq(a.distance2(&b), 25.);
        assert_approx_eq(b.distance2(&a), 25.);
    }

    #[test]
    fn vector_distance_i32() {
        const A: Vec2i = Vec2i::new(0, 0);
        const B: Vec2i = Vec2i::new(3, 4);

        assert_approx_eq(A.distance(&B), 5.);
        assert_approx_eq(B.distance(&A), 5.);

        assert_approx_eq(A.distance2(&B), 25.);
        assert_approx_eq(B.distance2(&A), 25.);
    }

    #[test]
    fn vector_distance_u32() {
        let a = Vec2u::new(0, 0);
        const B: Vec2u = Vec2u::new(3, 4);

        assert_approx_eq(a.distance(&B), 5.);
        assert_approx_eq(B.distance(&a), 5.);

        assert_approx_eq(a.distance2(&B), 25.);
        assert_approx_eq(B.distance2(&a), 25.);
    }
}
