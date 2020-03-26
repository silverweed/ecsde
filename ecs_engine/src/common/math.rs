use std::ops::{Add, Mul, Sub};

pub fn fast_invsqrt(n: f32) -> f32 {
    assert!(n != 0., "fast_invsqrt: argument cannot be 0!");
    let x2: f32 = n * 0.5;
    let mut i: u32 = n.to_bits();
    i = 0x5f37_5a86 - (i >> 1);
    let y: f32 = f32::from_bits(i);
    let y = y * (1.5 - (x2 * y * y));
    y * (1.5 - (x2 * y * y))
}

pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
    debug_assert!(min <= max);
    x.min(max).max(min)
}

// @WaitForStable: make this const when trait bounds are stable
pub fn lerp<T: Lerpable>(a: T, b: T, t: T) -> T {
    a * (T::ONE - t) + b * t
}

pub trait Lerpable: Copy + Add<Output = Self> + Mul<Output = Self> + Sub<Output = Self> {
    const ONE: Self;
}

macro_rules! def_lerpable {
    ($($type: ty),*: $one: expr) => {
        $(
            impl Lerpable for $type {
                const ONE: Self = $one;
            }
        )*
    }
}

def_lerpable!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize: 1);
def_lerpable!(f32, f64: 1.0);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_invsqrt() {
        // @Robustness: these tests use pretty senseless numbers as epsilon values.
        assert_approx_eq!(fast_invsqrt(2.), 0.707_106, eps = 0.000_01);
        assert_approx_eq!(fast_invsqrt(10000.), 0.01, eps = 0.000_01);
        assert_approx_eq!(fast_invsqrt(0.0001), 100., eps = 0.001);
        assert_approx_eq!(fast_invsqrt(1.), 1., eps = 0.000_01);
    }

    #[test]
    #[should_panic]
    fn fast_invsqrt_zero() {
        let _ = fast_invsqrt(0.);
    }
}
