// @WaitForStable: make most functions here const when possible.

#[inline(always)]
pub fn fast_sqrt(n: f32) -> f32 {
    n * fast_invsqrt_unchecked(n)
}

#[inline(always)]
pub fn fast_invsqrt(n: f32) -> f32 {
    assert!(n != 0., "fast_invsqrt: argument cannot be 0!");
    fast_invsqrt_unchecked(n)
}

/// Like fast_invsqrt() but does not check that n is non-zero
#[inline(always)]
pub fn fast_invsqrt_unchecked(n: f32) -> f32 {
    let x2: f32 = n * 0.5;
    let mut i: u32 = n.to_bits();
    i = 0x5f37_5a86 - (i >> 1);
    let y: f32 = f32::from_bits(i);
    let y = y * (1.5 - (x2 * y * y));
    y * (1.5 - (x2 * y * y))
}

#[inline]
pub fn clamp<T>(x: T, min: T, max: T) -> T
where
    T: PartialOrd,
{
    debug_assert!(min <= max);
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

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

    #[test]
    fn test_fast_sqrt() {
        // @Robustness: these tests use pretty senseless numbers as epsilon values.
        assert_approx_eq!(fast_sqrt(2.), 1.414_213, eps = 0.000_1);
        assert_approx_eq!(fast_sqrt(10000.), 100., eps = 0.001);
        assert_approx_eq!(fast_sqrt(32.345), 32.345f32.sqrt(), eps = 0.000_1);
        assert_approx_eq!(fast_sqrt(0.), 0., eps = 0.0);
    }
}
