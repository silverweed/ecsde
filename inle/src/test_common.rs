// Note: if the following line is uncommented, dependant crates won't import the module
// correctly. Investigate on this.
//#![cfg(test)]

#[rustfmt::skip]
#[cfg(test)]
use {
    crate::core::env::Env_Info,
    crate::resources::audio::Audio_Resources,
    crate::resources::gfx::Gfx_Resources,
};

#[cfg(test)]
// Used for setting up tests which need resources
pub fn create_test_resources_and_env<'a>() -> (Gfx_Resources<'a>, Audio_Resources<'a>, Env_Info) {
    let gfx = Gfx_Resources::new();
    let audio = Audio_Resources::new();
    let env = Env_Info::gather().expect("Failed to gather env info!");
    (gfx, audio, env)
}

#[cfg(test)]
pub trait Approx_Eq_Testable {
    fn cmp_list(&self) -> Vec<f32>;
}

#[cfg(test)]
impl Approx_Eq_Testable for f32 {
    fn cmp_list(&self) -> Vec<f32> {
        vec![*self]
    }
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_approx_eq {
    ($a: expr, $b: expr, eps = $eps: expr) => {{
        use float_cmp::ApproxEq;
        let list_a = $crate::test_common::Approx_Eq_Testable::cmp_list(&$a);
        let list_b = $crate::test_common::Approx_Eq_Testable::cmp_list(&$b);
        for (&xa, &xb) in list_a.iter().zip(list_b.iter()) {
            assert!(xa.approx_eq(xb, ($eps, 2)), "Expected: {}, Got: {}", xb, xa);
        }
    }};
    ($a: expr, $b: expr) => {
        assert_approx_eq!($a, $b, eps = 0.0);
    };
}
