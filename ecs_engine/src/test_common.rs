// Note: if the following line is uncommented, dependant crates won't import the module
// correctly. Investigate on this.
//#![cfg(test)]

use crate::core::env::Env_Info;
use crate::resources::audio::Audio_Resources;
use crate::resources::gfx::Gfx_Resources;
#[cfg(test)]
use float_cmp::ApproxEq;

// Used for setting up tests which need resources
pub fn create_test_resources_and_env<'a>() -> (Gfx_Resources<'a>, Audio_Resources<'a>, Env_Info) {
    let gfx = Gfx_Resources::new();
    let audio = Audio_Resources::new();
    let env = Env_Info::gather().expect("Failed to gather env info!");
    (gfx, audio, env)
}

#[cfg(test)]
pub fn assert_approx_eq(a: f32, b: f32) {
    assert!(a.approx_eq(b, (0.0, 2)), "Expected: {}, Got: {}", b, a);
}

#[cfg(test)]
pub fn assert_approx_eq_eps(a: f32, b: f32, eps: f32) {
    assert!(a.approx_eq(b, (eps, 2)), "Expected: {}, Got: {}", b, a);
}
