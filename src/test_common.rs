use crate::audio::sound_loader::Sound_Loader;
use crate::core::app::Resource_Loaders;
use crate::core::env::Env_Info;
use crate::gfx::window;
use crate::resources::Resources;
use sdl2::ttf::Sdl2TtfContext;

pub fn create_resource_loaders() -> (Resource_Loaders, sdl2::Sdl, sdl2::VideoSubsystem) {
    let sdl = sdl2::init().unwrap();
    let sdl_video = sdl.video().unwrap();
    (
        Resource_Loaders {
            texture_creator: window::create_render_canvas(&sdl_video, (0, 0), "").texture_creator(),
            ttf_context: sdl2::ttf::init().unwrap(),
            sound_loader: Sound_Loader {},
        },
        sdl,
        sdl_video,
    )
}

// Used for setting up tests which need resources
pub fn create_test_resources_and_env<'a>(
    loaders: &'a Resource_Loaders,
) -> (Resources<'a>, Env_Info) {
    let rsrc = Resources::new(
        &loaders.texture_creator,
        &loaders.ttf_context,
        &loaders.sound_loader,
    );
    let env = Env_Info::gather().expect("Failed to gather env info!");
    (rsrc, env)
}
