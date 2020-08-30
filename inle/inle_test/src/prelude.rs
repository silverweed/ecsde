#[macro_export]
macro_rules! assert_approx_eq {
    ($a: expr, $b: expr, eps = $eps: expr) => {{
        use $crate::float_cmp::ApproxEq;
        let list_a = $crate::approx_eq_testable::Approx_Eq_Testable::cmp_list(&$a);
        let list_b = $crate::approx_eq_testable::Approx_Eq_Testable::cmp_list(&$b);
        for (&xa, &xb) in list_a.iter().zip(list_b.iter()) {
            assert!(xa.approx_eq(xb, ($eps, 2)), "Expected: {}, Got: {}", xb, xa);
        }
    }};
    ($a: expr, $b: expr) => {
        assert_approx_eq!($a, $b, eps = 0.0);
    };
}
