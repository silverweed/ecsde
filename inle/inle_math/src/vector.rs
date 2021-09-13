use super::angle::Angle;
use super::math;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[cfg(feature = "gfx-sfml")]
mod sfml;

#[repr(C)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

pub type Vec2u = Vector2<u32>;
pub type Vec2f = Vector2<f32>;
pub type Vec2i = Vector2<i32>;

impl<T: Hash> Hash for Vector2<T> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.x.hash(state);
        self.y.hash(state);
    }
}

#[inline(always)]
pub fn lerp_v(v1: Vec2f, v2: Vec2f, t: f32) -> Vec2f {
    v2!(math::lerp(v1.x, v2.x, t), math::lerp(v1.y, v2.y, t))
}

impl<T: Copy> From<(T, T)> for Vector2<T> {
    fn from((x, y): (T, T)) -> Self {
        Self::new(x, y)
    }
}

impl<T: Copy> From<Vector2<T>> for (T, T) {
    fn from(v: Vector2<T>) -> Self {
        (v.x, v.y)
    }
}

impl From<Vec2u> for Vec2f {
    fn from(v: Vec2u) -> Self {
        Self::new(v.x as _, v.y as _)
    }
}

impl From<Vec2u> for Vec2i {
    fn from(v: Vec2u) -> Self {
        Self::new(v.x as _, v.y as _)
    }
}

impl From<Vec2i> for Vec2f {
    fn from(v: Vec2i) -> Self {
        Self::new(v.x as _, v.y as _)
    }
}

impl From<Vec2i> for Vec2u {
    fn from(v: Vec2i) -> Self {
        Self::new(v.x as _, v.y as _)
    }
}

impl From<Vec2f> for Vec2u {
    fn from(v: Vec2f) -> Self {
        Self::new(v.x as _, v.y as _)
    }
}

impl From<Vec2f> for Vec2i {
    fn from(v: Vec2f) -> Self {
        Self::new(v.x as _, v.y as _)
    }
}

impl<T> From<Vector3<T>> for Vector2<T> {
    fn from(v: Vector3<T>) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl<T> Vector2<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: ToString> std::string::ToString for Vector2<T> {
    fn to_string(&self) -> String {
        format!("{}, {}", self.x.to_string(), self.y.to_string())
    }
}

const NORMALIZE_EPSILON: f32 = 0.000_001;

impl<T> Vector2<T>
where
    T: Copy
        + Add<Output = T>
        + Mul<Output = T>
        + std::convert::Into<f32>
        + std::convert::From<f32>
        + Default,
{
    #[inline]
    pub fn magnitude2(self) -> T {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn magnitude(self) -> f32 {
        self.magnitude2().into().sqrt()
    }

    #[inline]
    pub fn magnitude_fast(self) -> f32 {
        math::fast_sqrt(self.magnitude2().into())
    }

    #[inline]
    /// Like `normalized_or_zero` but panics if length is 0.
    pub fn normalized(self) -> Self {
        let mag = self.magnitude2().into();
        debug_assert!(mag > 0.);
        let den = 1.0 / mag.sqrt();
        Self {
            x: T::from(self.x.into() * den),
            y: T::from(self.y.into() * den),
        }
    }

    #[inline]
    /// Like `normalized` but faster and less precise
    pub fn normalized_fast(self) -> Self {
        let mag = self.magnitude2().into();
        debug_assert!(mag > 0.);
        let den = math::fast_invsqrt(mag);
        Self {
            x: T::from(self.x.into() * den),
            y: T::from(self.y.into() * den),
        }
    }

    #[inline]
    /// Returns the normalized vector, or 0 if it has length 0.
    pub fn normalized_or_zero(self) -> Self {
        let mag = self.magnitude2().into();
        if mag == 0. {
            return Self::default();
        }

        let den = 1.0 / mag.sqrt();
        Self {
            x: T::from(self.x.into() * den),
            y: T::from(self.y.into() * den),
        }
    }

    #[inline]
    /// Like `normalized_or_zero` but faster and less precise
    pub fn normalized_or_zero_fast(self) -> Self {
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

    #[inline]
    pub fn is_normalized(self) -> bool {
        (self.magnitude2().into() - 1.0).abs() < NORMALIZE_EPSILON
    }

    #[inline]
    pub fn is_normalized_or_zero(self) -> bool {
        let mag = self.magnitude2().into();
        mag == 0. || (mag - 1.0).abs() < NORMALIZE_EPSILON
    }

    #[inline]
    pub fn rotated(self, angle: Angle) -> Self {
        let rads = angle.as_rad();
        let x = self.x.into();
        let y = self.y.into();
        let (s, c) = rads.sin_cos();
        Self {
            x: T::from(c * x - s * y),
            y: T::from(s * x + c * y),
        }
    }

    #[inline]
    pub fn dot(self, b: Self) -> T {
        self.x * b.x + self.y * b.y
    }
}

#[cfg(debug_assertions)]
#[inline(always)]
pub fn sanity_check_v(v: Vec2f) {
    debug_assert!(!v.x.is_nan());
    debug_assert!(!v.y.is_nan());
}

#[cfg(not(debug_assertions))]
pub fn sanity_check_v(_: Vec2f) {}

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

// @Incomplete @WaitForStable: we should really implement Mul<Vector2<T>> for T, but that's not
// currently allowed, as far as I understand.
impl Mul<Vector2<f32>> for f32 {
    type Output = Vector2<f32>;

    fn mul(self, other: Vector2<f32>) -> Self::Output {
        Self::Output {
            x: self * other.x,
            y: self * other.y,
        }
    }
}

impl<T: Copy + Mul<Output = T>> Mul for Vector2<T> {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
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

impl<T: Copy + Div<Output = T>> Div for Vector2<T> {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
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

impl<T: Copy + Mul<Output = T>> MulAssign for Vector2<T> {
    fn mul_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x * other.x,
            y: self.y * other.y,
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

#[cfg(test)]
impl inle_test::approx_eq_testable::Approx_Eq_Testable for Vec2f {
    fn cmp_list(&self) -> Vec<f32> {
        vec![self.x, self.y]
    }
}

impl Vec2f {
    #[inline]
    pub fn from_polar(r: f32, theta: f32) -> Self {
        let (s, c) = theta.sin_cos();
        Self { x: r * c, y: r * s }
    }

    #[inline]
    pub fn from_rotation(rot: Angle) -> Self {
        let (s, c) = rot.as_rad().sin_cos();
        v2!(c, s)
    }

    // @WaitForStable: make this const as soon as sqrt() is stable as const
    pub fn distance(self, other: Self) -> f32 {
        self.distance2(other).sqrt()
    }

    // @WaitForStable: make this const as soon as `-` on f32 is stable as const
    pub fn distance2(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

impl Vec2i {
    pub fn distance(self, other: Self) -> f32 {
        self.distance2(other).sqrt()
    }

    pub const fn distance2(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy) as f32
    }
}

impl Vec2u {
    pub fn distance(self, other: Self) -> f32 {
        self.distance2(other).sqrt()
    }

    pub const fn distance2(self, other: Self) -> f32 {
        let dx = self.x as i32 - other.x as i32;
        let dy = self.y as i32 - other.y as i32;
        (dx * dx + dy * dy) as f32
    }
}

#[repr(C)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub type Vec3i = Vector3<i32>;
pub type Vec3u = Vector3<u32>;
pub type Vec3f = Vector3<f32>;

impl<T: Default> Default for Vector3<T> {
    fn default() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
            z: T::default(),
        }
    }
}

impl<T: Copy> Copy for Vector3<T> {}
impl<T: Clone> Clone for Vector3<T> {
    fn clone(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: self.y.clone(),
            z: self.z.clone(),
        }
    }
}

impl<T> Vector3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: Copy> From<&[T; 3]> for Vector3<T> {
    fn from(vals: &[T; 3]) -> Self {
        Self {
            x: vals[0],
            y: vals[1],
            z: vals[2],
        }
    }
}

impl<T> Vector3<T>
where
    T: Copy + Add<Output = T> + Mul<Output = T> + Sub<Output = T>,
{
    pub fn cross(&self, v: &Vector3<T>) -> Self {
        Self {
            x: self.y * v.z - self.z * v.y,
            y: self.z * v.x - self.x * v.z,
            z: self.x * v.y - self.y * v.x,
        }
    }
}

impl<T: ToString> std::string::ToString for Vector3<T> {
    fn to_string(&self) -> String {
        format!(
            "{}, {}, {}",
            self.x.to_string(),
            self.y.to_string(),
            self.z.to_string()
        )
    }
}

impl<T: Debug> Debug for Vector3<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{{ x: {:?}, y: {:?}, z: {:?}  }}",
            self.x, self.y, self.z
        )
    }
}

impl<T: PartialEq> PartialEq for Vector3<T> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl<T: Eq> Eq for Vector3<T> {}

impl<T: Copy> std::ops::Index<usize> for Vector3<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        match idx {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => fatal!("Tried to index Vector3 with invalid index {}", idx),
        }
    }
}

#[cfg(test)]
impl inle_test::approx_eq_testable::Approx_Eq_Testable for Vec3f {
    fn cmp_list(&self) -> Vec<f32> {
        vec![self.x, self.y, self.z]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec2_default() {
        assert_eq!(Vec2u::default(), Vec2u::new(0, 0));
        assert_eq!(Vec2i::default(), Vec2i::new(0, 0));
        assert_eq!(Vec2f::default(), Vec2f::new(0., 0.));
    }

    #[test]
    fn vec2_copy() {
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
    fn vec2_add() {
        assert_eq!(Vec2i::new(0, 1) + Vec2i::new(3, 1), Vec2i::new(3, 2));
        assert_eq!(
            Vec2u::new(2, 10) + Vec2u::new(103, 100),
            Vec2u::new(105, 110)
        );
        assert_eq!(Vec2f::new(5., 0.) + Vec2f::new(-2., 9.), Vec2f::new(3., 9.));
    }

    #[test]
    fn vec2_sub() {
        assert_eq!(Vec2i::new(0, 1) - Vec2i::new(3, 1), Vec2i::new(-3, 0));
        assert_eq!(Vec2u::new(200, 10) - Vec2u::new(103, 10), Vec2u::new(97, 0));
        assert_eq!(
            Vec2f::new(5., 0.) - Vec2f::new(-2., 9.),
            Vec2f::new(7., -9.)
        );
    }

    #[test]
    fn vec2_mul_scalar() {
        assert_eq!(Vec2i::new(0, 3) * 3, Vec2i::new(0, 9));
        assert_eq!(Vec2u::new(200, 10) * 2, Vec2u::new(400, 20));
        assert_eq!(Vec2f::new(5., 0.) * 0.5, Vec2f::new(2.5, 0.));
    }

    #[test]
    fn vec2_mul_compwise() {
        assert_eq!(Vec2i::new(0, 3) * Vec2i::new(2, 3), Vec2i::new(0, 9));
        assert_eq!(
            Vec2u::new(200, 10) * Vec2u::new(2, 10),
            Vec2u::new(400, 100)
        );
        assert_eq!(
            Vec2f::new(5., 0.1) * Vec2f::new(0.5, 1.),
            Vec2f::new(2.5, 0.1)
        );
    }

    #[test]
    fn vec2_div_compwise() {
        assert_eq!(Vec2i::new(0, 3) / Vec2i::new(2, 3), Vec2i::new(0, 1));
        assert_eq!(Vec2u::new(200, 10) / Vec2u::new(2, 5), Vec2u::new(100, 2));
        assert_eq!(
            Vec2f::new(5., 0.1) / Vec2f::new(0.5, 1.),
            Vec2f::new(10., 0.1)
        );
    }

    #[test]
    fn vec2_div() {
        assert_eq!(Vec2i::new(0, 3) / 3, Vec2i::new(0, 1));
        assert_eq!(Vec2u::new(200, 10) / 2, Vec2u::new(100, 5));
        assert_eq!(Vec2f::new(5., 0.) / 0.5, Vec2f::new(10., 0.));
    }

    #[test]
    fn vec2_neg() {
        assert_eq!(-Vec2i::new(10, 3), Vec2i::new(-10, -3));
        assert_eq!(-Vec2f::new(5., 0.5), Vec2f::new(-5., -0.5));
    }

    #[test]
    fn vec2_add_assign() {
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
    fn vec2_sub_assign() {
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
    fn vec2_mul_assign() {
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
    fn vec2_div_assign() {
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
    fn vec2_normalize() {
        let v = Vec2f::new(1., 1.).normalized_or_zero();
        assert_approx_eq!(v.x, 0.707_106_7);
        assert_eq!(v.x, v.y);
        assert!(v.is_normalized());

        let v = Vec2f::new(1., 1.).normalized();
        assert_approx_eq!(v.x, 0.707_106_7);
        assert_eq!(v.x, v.y);
        assert!(v.is_normalized());

        assert_eq!(Vec2f::new(0., 0.).normalized_or_zero(), Vec2f::new(0., 0.));
        assert!(Vec2f::new(0., 0.).is_normalized_or_zero());
        assert!(!Vec2f::new(0., 0.).is_normalized());
    }

    #[test]
    #[should_panic]
    fn vec2_normalize_zero_length() {
        let _ = Vec2f::new(0., 0.).normalized();
    }

    #[test]
    fn vec2_distance_f32() {
        let a = Vec2f::new(0., 0.);
        let b = Vec2f::new(3., 4.);

        assert_approx_eq!(a.distance(b), 5.);
        assert_approx_eq!(b.distance(a), 5.);

        assert_approx_eq!(a.distance2(b), 25.);
        assert_approx_eq!(b.distance2(a), 25.);
    }

    #[test]
    fn vec2_distance_i32() {
        const A: Vec2i = Vec2i::new(0, 0);
        const B: Vec2i = Vec2i::new(3, 4);

        assert_approx_eq!(A.distance(B), 5.);
        assert_approx_eq!(B.distance(A), 5.);

        assert_approx_eq!(A.distance2(B), 25.);
        assert_approx_eq!(B.distance2(A), 25.);
    }

    #[test]
    fn vec2_distance_u32() {
        let a = Vec2u::new(0, 0);
        const B: Vec2u = Vec2u::new(3, 4);

        assert_approx_eq!(a.distance(B), 5.);
        assert_approx_eq!(B.distance(a), 5.);

        assert_approx_eq!(a.distance2(B), 25.);
        assert_approx_eq!(B.distance2(a), 25.);
    }

    #[test]
    fn vec3_default() {
        assert_eq!(Vec3u::default(), Vec3u::new(0, 0, 0));
        assert_eq!(Vec3i::default(), Vec3i::new(0, 0, 0));
        assert_eq!(Vec3f::default(), Vec3f::new(0., 0., 0.));
    }

    #[test]
    fn vec3_copy() {
        let a = Vec3i::new(2, 3, 4);
        let b = a;
        assert_eq!(a, b);

        let a = Vec3u::new(2, 3, 4);
        let b = a;
        assert_eq!(a, b);

        let a = Vec3f::new(2., 3., 4.);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn vec3_cross() {
        let a = Vec3i::new(1, 2, 3);
        let b = Vec3i::new(4, 5, 6);

        assert_eq!(
            a.cross(&b),
            Vec3i::new(2 * 6 - 3 * 5, 3 * 4 - 1 * 6, 1 * 5 - 2 * 4)
        );
    }
}
