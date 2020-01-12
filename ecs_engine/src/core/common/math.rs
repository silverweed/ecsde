pub fn fast_invsqrt(n: f32) -> f32 {
    const THREE_HALFS: f32 = 1.5;

    let x2: f32 = n * 0.5;
    let mut i: u32 = unsafe { std::mem::transmute(n) };
    i = 0x5f37_5a86 - (i >> 1);
    let y: f32 = f32::from_bits(i);
    let y = y * (THREE_HALFS - (x2 * y * y));
    y * (THREE_HALFS - (x2 * y * y))
}

#[test]
fn test_fast_invsqrt() {
    use crate::test_common::assert_approx_eq_eps;

    // @Robustness: these tests use pretty senseless numbers as epsilon values.
    assert_approx_eq_eps(fast_invsqrt(2.), 0.707_106, 0.000_01);
    assert_approx_eq_eps(fast_invsqrt(10000.), 0.01, 0.000_01);
    assert_approx_eq_eps(fast_invsqrt(0.0001), 100., 0.001);
    assert_approx_eq_eps(fast_invsqrt(1.), 1., 0.000_01);
}
