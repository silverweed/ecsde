#[rustfmt::skip]
use {
    inle_core::env::Env_Info,
    inle_resources::audio::Audio_Resources,
    inle_resources::gfx::Gfx_Resources,
};

// Used for setting up tests which need resources
pub fn create_test_resources_and_env<'a>() -> (Gfx_Resources<'a>, Audio_Resources<'a>, Env_Info) {
    let mut gfx = Gfx_Resources::new();
    let audio = Audio_Resources::new();
    let env = Env_Info::gather().expect("Failed to gather env info!");
    gfx.init();
    (gfx, audio, env)
}

#[cfg(feature = "gfx-gl")]
pub fn load_gl_pointers() -> (glfw::Window, glfw::Glfw) {
    use glfw::Context;

    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let (mut window, _) = glfw
        .create_window(1, 1, "", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");
    window.make_current();
    gl::load_with(|symbol| window.get_proc_address(symbol));
    (window, glfw)
}
