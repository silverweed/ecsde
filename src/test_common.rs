use crate::core::env::Env_Info;
use crate::resources::audio::Audio_Resources;
use crate::resources::gfx::Gfx_Resources;

#[cfg(feature = "use-sdl")]
use sdl2::ttf::Sdl2TtfContext;

#[cfg(feature = "use-sdl")]
pub fn create_resource_loaders() -> (Resource_Loaders, sdl2::Sdl, sdl2::VideoSubsystem) {
    let sdl = sdl2::init().unwrap();
    let sdl_video = sdl.video().unwrap();
    (
        Resource_Loaders {
            texture_creator: window::create_render_window(&sdl_video, (0, 0), "").texture_creator(),
            ttf_context: sdl2::ttf::init().unwrap(),
            sound_loader: Sound_Loader {},
        },
        sdl,
        sdl_video,
    )
}

// Used for setting up tests which need resources
pub fn create_test_resources_and_env<'a>() -> (Gfx_Resources<'a>, Audio_Resources<'a>, Env_Info) {
    let gfx = Gfx_Resources::new();
    let audio = Audio_Resources::new();
    let env = Env_Info::gather().expect("Failed to gather env info!");
    (gfx, audio, env)
}
