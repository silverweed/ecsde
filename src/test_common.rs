use crate::core::env::Env_Info;
use crate::gfx::window;
use crate::resources::Resources;

// Used for setting up tests which need resources
pub fn create_test_resources_and_env() -> (Resources, Env_Info) {
    let sdl = sdl2::init().unwrap();
    let sdl_video = sdl.video().unwrap();
    let texture_creator = window::create_render_canvas(&sdl_video, (0, 0), "").texture_creator();
    let rsrc = Resources::new(texture_creator);
    let env = Env_Info::gather().expect("Failed to gather env info!");
    (rsrc, env)
}
