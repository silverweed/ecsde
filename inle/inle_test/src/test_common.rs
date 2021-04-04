#[rustfmt::skip]
use {
    inle_core::env::Env_Info,
    inle_resources::audio::Audio_Resources,
    inle_resources::gfx::Gfx_Resources,
};

// @FIXME: load_texture is not usable unless we load the gl functions!
// Used for setting up tests which need resources
pub fn create_test_resources_and_env<'a>() -> (Gfx_Resources<'a>, Audio_Resources<'a>, Env_Info) {
    let mut gfx = Gfx_Resources::new();
    let audio = Audio_Resources::new();
    let env = Env_Info::gather().expect("Failed to gather env info!");
    gfx.init();
    (gfx, audio, env)
}

pub fn load_gl_pointers() -> glfw::Glfw {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let (mut window, _) = glfw
        .create_window(1, 1, "", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");
    gl::load_with(|symbol| window.get_proc_address(symbol));
    glfw
}
