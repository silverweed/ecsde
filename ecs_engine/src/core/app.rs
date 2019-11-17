use super::app_config::App_Config;
use super::common::colors;
use super::common::Maybe_Error;
use super::env::Env_Info;
use super::time;
use crate::cfg::{self, Cfg_Var};
use crate::core::systems::Core_Systems;
use crate::fs;
use crate::gfx;
use crate::gfx::align;
use crate::input;
use crate::replay::{replay_data, replay_input_provider};
use crate::resources;
use notify::RecursiveMode;
use std::time::Duration;

#[cfg(debug_assertions)]
use super::common::stringid::String_Id;
#[cfg(debug_assertions)]
use crate::core::systems::Debug_Systems;

#[cfg(debug_assertions)]
use crate::debug;

pub struct Engine_State<'r> {
    pub should_close: bool,

    pub env: Env_Info,
    pub config: cfg::Config,
    pub app_config: App_Config,

    pub time: time::Time,

    pub systems: Core_Systems,
    #[cfg(debug_assertions)]
    pub debug_systems: Debug_Systems,

    pub replay_data: Option<replay_data::Replay_Data>,

    pub gfx_resources: resources::gfx::Gfx_Resources<'r>,
    pub audio_resources: resources::audio::Audio_Resources<'r>,
}

pub fn create_engine_state<'r>(app_config: App_Config) -> Engine_State<'r> {
    let env = Env_Info::gather().unwrap();
    let config = cfg::Config::new_from_dir(env.get_cfg_root());
    let systems = Core_Systems::new(&env);
    #[cfg(debug_assertions)]
    let debug_systems = Debug_Systems::new(&config);
    let time = time::Time::new();
    let replay_data = maybe_create_replay_data(&app_config);
    let gfx_resources = resources::gfx::Gfx_Resources::new();
    let audio_resources = resources::audio::Audio_Resources::new();

    Engine_State {
        should_close: false,
        env,
        config,
        time,
        app_config,
        gfx_resources,
        audio_resources,
        systems,
        #[cfg(debug_assertions)]
        debug_systems,
        replay_data,
    }
}

pub fn start_config_watch(env: &Env_Info, config: &mut cfg::Config) -> Maybe_Error {
    let config_watcher = Box::new(cfg::sync::Config_Watch_Handler::new(config));
    let config_watcher_cfg = fs::file_watcher::File_Watch_Config {
        interval: Duration::from_secs(1),
        recursive_mode: RecursiveMode::Recursive,
    };
    fs::file_watcher::start_file_watch(
        env.get_cfg_root().to_path_buf(),
        config_watcher_cfg,
        vec![config_watcher],
    )?;
    Ok(())
}

pub fn init_engine_systems(engine_state: &mut Engine_State) -> Maybe_Error {
    let systems = &mut engine_state.systems;

    systems.input_system.init()?;

    Ok(())
}

#[cfg(debug_assertions)]
pub fn start_recording(engine_state: &mut Engine_State) -> Maybe_Error {
    if engine_state.replay_data.is_none()
        && Cfg_Var::<bool>::new("engine/debug/replay/record", &engine_state.config)
            .read(&engine_state.config)
    {
        engine_state
            .debug_systems
            .replay_recording_system
            .start_recording_thread(&engine_state.config)?;
    }

    Ok(())
}

#[cfg(debug_assertions)]
pub fn init_engine_debug(engine_state: &mut Engine_State<'_>) -> Maybe_Error {
    use crate::core::common::vector::Vec2f;
    use debug::{fadeout_overlay, overlay};

    const FONT: &str = "Hack-Regular.ttf";

    let font = engine_state
        .gfx_resources
        .load_font(&resources::gfx::font_path(&engine_state.env, FONT));

    // @Robustness: add font validity check

    let (target_win_size_x, target_win_size_y) = (
        engine_state.app_config.target_win_size.0 as f32,
        engine_state.app_config.target_win_size.1 as f32,
    );
    let debug_ui_system = &mut engine_state.debug_systems.debug_ui_system;

    // Debug overlays
    {
        let mut debug_overlay_config = overlay::Debug_Overlay_Config {
            row_spacing: 2.0,
            font_size: 20,
            pad_x: 5.0,
            pad_y: 5.0,
            background: colors::rgba(25, 25, 25, 210),
        };

        let mut joy_overlay = debug_ui_system.create_overlay(
            String_Id::from("joysticks"),
            debug_overlay_config,
            font,
        );
        joy_overlay.horiz_align = align::Align::End;
        joy_overlay.position = Vec2f::new(target_win_size_x, 0.0);

        debug_overlay_config.font_size = 16;
        let mut time_overlay =
            debug_ui_system.create_overlay(String_Id::from("time"), debug_overlay_config, font);
        time_overlay.horiz_align = align::Align::End;
        time_overlay.vert_align = align::Align::End;
        time_overlay.position = Vec2f::new(target_win_size_x, target_win_size_y);

        let mut fps_overlay =
            debug_ui_system.create_overlay(String_Id::from("fps"), debug_overlay_config, font);
        fps_overlay.vert_align = align::Align::End;
        fps_overlay.position = Vec2f::new(0.0, target_win_size_y as f32);
    }

    // Debug fadeout overlays
    {
        let fadeout_overlay_config = fadeout_overlay::Fadeout_Debug_Overlay_Config {
            row_spacing: 2.0,
            font_size: 20,
            pad_x: 5.0,
            pad_y: 5.0,
            background: colors::rgba(25, 25, 25, 210),
            fadeout_time: Duration::from_secs(3),
            max_rows: 30,
        };

        let mut fadeout_overlay = debug_ui_system.create_fadeout_overlay(
            String_Id::from("msg"),
            fadeout_overlay_config,
            font,
        );
        fadeout_overlay.horiz_align = align::Align::Begin;
        fadeout_overlay.position = Vec2f::new(0.0, 0.0);
    }

    Ok(())
}

pub fn create_input_provider(
    replay_data: &mut Option<replay_data::Replay_Data>,
    cfg: &cfg::Config,
) -> Box<dyn input::provider::Input_Provider> {
    // Consumes self.replay_data!
    let replay_data = replay_data.take();
    if let Some(replay_data) = replay_data {
        let config = replay_input_provider::Replay_Input_Provider_Config {
            disable_input_during_replay: Cfg_Var::new(
                "engine/debug/replay/disable_input_during_replay",
                cfg,
            ),
        };
        Box::new(replay_input_provider::Replay_Input_Provider::new(
            config,
            replay_data,
        ))
    } else {
        Box::new(input::default_input_provider::Default_Input_Provider::default())
    }
}

pub fn handle_core_actions(
    actions: &[input::core_actions::Core_Action],
    window: &mut gfx::window::Window_Handle,
) -> bool {
    use input::core_actions::Core_Action;

    for action in actions.iter() {
        match action {
            Core_Action::Quit => return true,
            Core_Action::Resize(new_width, new_height) => {
                gfx::window::resize_keep_ratio(window, *new_width, *new_height)
            }
        }
    }

    false
}

#[cfg(debug_assertions)]
fn maybe_create_replay_data(cfg: &App_Config) -> Option<replay_data::Replay_Data> {
    if let Some(path) = &cfg.replay_file {
        match replay_data::Replay_Data::from_file(&path) {
            Ok(data) => Some(data),
            Err(err) => {
                eprintln!(
                    "[ ERROR ] Failed to load replay data from {:?}: {}",
                    path, err
                );
                None
            }
        }
    } else {
        None
    }
}

#[cfg(not(debug_assertions))]
fn maybe_create_replay_data(cfg: &App_Config) -> Option<replay_data::Replay_Data> {
    None
}
