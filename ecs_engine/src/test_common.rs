// Note: if the following line is uncommented, dependant crates won't import the module
// correctly. Investigate on this.
//#![cfg(test)]

#[rustfmt::skip]
#[cfg(test)]
use {
    crate::core::env::Env_Info,
    crate::resources::audio::Audio_Resources,
    crate::resources::gfx::Gfx_Resources,
    float_cmp::ApproxEq,
    std::iter::Iterator,
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
pub fn assert_approx_eq<T: Approx_Eq_Testable>(a: T, b: T) {
    assert_approx_eq_eps(a, b, 0.0);
}

#[cfg(test)]
pub fn assert_approx_eq_eps<T: Approx_Eq_Testable>(a: T, b: T, eps: f32) {
    for (&xa, &xb) in a.cmp_list().iter().zip(b.cmp_list().iter()) {
        assert!(xa.approx_eq(xb, (eps, 2)), "Expected: {}, Got: {}", xb, xa);
    }
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
