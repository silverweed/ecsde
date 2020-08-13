use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Copy, Clone, Default)]
pub struct Angle(f32); // The wrapped angle is in radians.

const PI: f32 = std::f32::consts::PI;
const TAU: f32 = 2.0 * PI;

impl PartialEq for Angle {
    fn eq(&self, other: &Self) -> bool {
        self.as_rad_0tau() == other.as_rad_0tau()
    }
}

impl PartialOrd for Angle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_rad_0tau().partial_cmp(&other.as_rad_0tau())
    }
}

impl Neg for Angle {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Angle(-self.0)
    }
}

impl Add for Angle {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Angle(self.0 + other.0)
    }
}

impl Sub for Angle {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Angle(self.0 - other.0)
    }
}

impl Mul<f32> for Angle {
    type Output = Self;
    fn mul(self, other: f32) -> Self::Output {
        Angle(self.0 * other)
    }
}

impl Div<f32> for Angle {
    type Output = Self;
    fn div(self, other: f32) -> Self::Output {
        Angle(self.0 / other)
    }
}

impl Div for Angle {
    type Output = f32;
    fn div(self, other: Angle) -> f32 {
        self.0 / other.0
    }
}

impl AddAssign for Angle {
    fn add_assign(&mut self, other: Self) {
        *self = Angle(self.0 + other.0)
    }
}

impl SubAssign for Angle {
    fn sub_assign(&mut self, other: Self) {
        *self = Angle(self.0 - other.0)
    }
}

impl MulAssign<f32> for Angle {
    fn mul_assign(&mut self, other: f32) {
        *self = Angle(self.0 * other)
    }
}

impl DivAssign<f32> for Angle {
    fn div_assign(&mut self, other: f32) {
        *self = Angle(self.0 / other)
    }
}

impl fmt::Debug for Angle {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Rad({})", self.0)
    }
}

pub const fn rad(r: f32) -> Angle {
    Angle(r)
}

// @WaitForStable: make this const as soon as stabilization lands
pub fn deg(d: f32) -> Angle {
    Angle(deg2rad(d))
}

// @WaitForStable: make this const as soon as stabilization lands
pub fn deg2rad(deg: f32) -> f32 {
    deg * 0.0174_5329_2
}

// @WaitForStable: make this const as soon as stabilization lands
pub fn rad2deg(rad: f32) -> f32 {
    rad * 57.2957_8
}

impl Angle {
    pub const fn as_rad(self) -> f32 {
        self.0
    }

    // @WaitForStable: make this const as soon as stabilization lands
    pub fn as_deg(self) -> f32 {
        rad2deg(self.0)
    }

    /// Returns the angle between [-PI, PI)
    pub fn as_rad_negpipi(self) -> f32 {
        let mut a = (self.0 + PI) % TAU;
        if a < 0. {
            a += TAU;
        }
        a - PI
    }

    /// Returns the angle between [0-TAU)
    pub fn as_rad_0tau(self) -> f32 {
        let mut a = self.0 % TAU;
        if a < 0. {
            a += TAU;
        }
        a
    }

    /// Returns the angle between [-180, 180)
    pub fn as_deg_neg180180(self) -> f32 {
        rad2deg(self.as_rad_negpipi())
    }

    /// Returns the angle between [0-360)
    pub fn as_deg_0360(self) -> f32 {
        rad2deg(self.as_rad_0tau())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common::*;

    impl Approx_Eq_Testable for Angle {
        fn cmp_list(&self) -> Vec<f32> {
            vec![self.0]
        }
    }

    #[test]
    fn angle_construct() {
        assert_approx_eq!(rad(1.345).as_rad(), 1.345);
        assert_approx_eq!(deg(1.345).as_deg(), 1.345);
        assert_approx_eq!(deg(1.345).as_rad(), deg2rad(1.345));
        assert_approx_eq!(rad(1.345).as_deg(), rad2deg(1.345));
    }

    #[test]
    fn angle_conv() {
        assert_approx_eq!(rad(PI * 0.5), deg(90.));
        assert_approx_eq!(rad(TAU), deg(360.));
        assert_approx_eq!(rad(1.), rad(deg2rad(rad2deg(1.))));
        assert_approx_eq!(deg(1.), deg(rad2deg(deg2rad(1.))));
    }

    #[test]
    fn angle_neg() {
        assert_approx_eq!(-deg(200.), deg(-200.));
        assert_approx_eq!(-rad(PI), deg(-180.));
    }

    #[test]
    fn angle_sum() {
        assert_approx_eq!(deg(200.) + deg(200.), deg(400.));
        assert_approx_eq!(rad(PI) + deg(100.), deg(280.));
    }

    #[test]
    fn angle_diff() {
        assert_approx_eq!(rad(1.5) - rad(2.5), rad(-1.0));
        assert_approx_eq!(deg(180.) - rad(PI), rad(0.));
    }

    #[test]
    fn angle_product() {
        assert_approx_eq!(rad(1.5) * 3., rad(4.5));
        assert_approx_eq!(deg(90.) * 4., rad(2. * PI));
        assert_approx_eq!(deg(110.) * 0.1, deg(11.0));
    }

    #[test]
    fn angle_division() {
        assert_approx_eq!(rad(4.0) / 4., rad(1.));
        assert_approx_eq!(deg(90.) / 2., rad(PI * 0.25));
        assert_approx_eq!(deg(10.) / 0.1, deg(100.));
    }

    #[test]
    fn angle_ratio() {
        assert_approx_eq!(rad(PI) / rad(PI), 1.);
        assert_approx_eq!(deg(360.) / rad(PI * 0.5), 4.);
        assert_approx_eq!(deg(0.) / rad(15.), 0.);
        assert!((deg(0.) / rad(0.)).is_nan());
    }

    #[test]
    fn angle_add_assign() {
        let mut a = rad(PI);
        a += deg(90.);
        assert_approx_eq!(a, deg(270.));

        a += rad(0.);
        assert_approx_eq!(a, deg(270.));
    }

    #[test]
    fn angle_sub_assign() {
        let mut a = rad(PI);
        a -= deg(90.);
        assert_approx_eq!(a, deg(90.));

        a -= deg(0.);
        assert_approx_eq!(a, deg(90.));
    }

    #[test]
    fn angle_mul_assign() {
        let mut a = rad(PI);
        a *= 3.0;
        assert_approx_eq!(a, deg(540.));

        a *= 0.;
        assert_approx_eq!(a, rad(0.));
    }

    #[test]
    fn angle_div_assign() {
        let mut a = rad(TAU);
        a /= 2.;
        assert_approx_eq!(a, deg(180.));

        a /= 1.;
        assert_approx_eq!(a, rad(PI));
    }

    #[test]
    fn angle_normalize() {
        assert_approx_eq!(deg(0.).as_deg_0360(), 0.);
        assert_approx_eq!(deg(500.).as_deg_0360(), 140.);
        assert_approx_eq!(deg(500.).as_deg_neg180180(), 140.);
        assert_approx_eq!(deg(-500.).as_deg_0360(), 220.);
        assert_approx_eq!(deg(-500.).as_deg_neg180180(), -140.);
        assert_approx_eq!(rad(0.).as_rad_negpipi(), 0.);
        assert_approx_eq!(rad(TAU).as_rad_negpipi(), 0., eps = 0.000_001);
        assert_approx_eq!(rad(TAU).as_rad_0tau(), 0., eps = 0.000_001);
        assert_approx_eq!(deg(-10.).as_deg_neg180180(), -10.);
        assert_approx_eq!(deg(-190.).as_deg_neg180180(), 170.);
        assert_approx_eq!(deg(190.).as_deg_neg180180(), -170.);
        assert_approx_eq!(rad(3. * PI).as_rad_0tau(), PI);
    }
}
