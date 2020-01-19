use super::app_config::App_Config;
use super::common::Maybe_Error;
use super::env::Env_Info;
use super::time;
use crate::cfg;
use crate::core::systems::Core_Systems;
use crate::fs;
use crate::gfx;
use crate::input;
use crate::prelude::{new_debug_tracer, Debug_Tracer};
use notify::RecursiveMode;
use std::time::Duration;

#[cfg(debug_assertions)]
use {
    super::common::stringid::String_Id,
    crate::cfg::Cfg_Var,
    crate::core::systems::Debug_Systems,
    crate::debug,
    crate::replay::{replay_data, replay_input_provider},
    crate::resources::{self, gfx::Gfx_Resources},
};

pub struct Engine_State<'r> {
    pub should_close: bool,

    pub env: Env_Info,
    pub config: cfg::Config,
    pub app_config: App_Config,

    pub time: time::Time,

    pub systems: Core_Systems<'r>,

    pub tracer: Debug_Tracer,

    #[cfg(debug_assertions)]
    pub debug_systems: Debug_Systems,

    #[cfg(debug_assertions)]
    pub replay_data: Option<replay_data::Replay_Data>,
}

pub fn create_engine_state<'r>(
    env: Env_Info,
    config: cfg::Config,
    app_config: App_Config,
) -> Engine_State<'r> {
    let systems = Core_Systems::new(&env);
    let time = time::Time::new();
    #[cfg(debug_assertions)]
    let debug_systems = Debug_Systems::new(&config);

    Engine_State {
        should_close: false,
        env,
        config,
        app_config,
        time,
        systems,
        #[cfg(debug_assertions)]
        debug_systems,
        #[cfg(debug_assertions)]
        replay_data: None,
        tracer: new_debug_tracer(),
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
    engine_state.systems.input_system.init()?;

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
pub fn init_engine_debug(
    engine_state: &mut Engine_State<'_>,
    gfx_resources: &mut Gfx_Resources<'_>,
) -> Maybe_Error {
    use crate::core::common::colors;
    use crate::core::common::vector::Vec2f;
    use crate::gfx::align;
    use debug::{fadeout_overlay, overlay};

    const FONT: &str = "Hack-Regular.ttf";

    let font = gfx_resources.load_font(&resources::gfx::font_path(&engine_state.env, FONT));

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
            font_size: 14,
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

        debug_overlay_config.font_size = 13;
        let mut time_overlay =
            debug_ui_system.create_overlay(String_Id::from("time"), debug_overlay_config, font);
        time_overlay.horiz_align = align::Align::End;
        time_overlay.vert_align = align::Align::End;
        time_overlay.position = Vec2f::new(target_win_size_x, target_win_size_y);

        let mut fps_overlay =
            debug_ui_system.create_overlay(String_Id::from("fps"), debug_overlay_config, font);
        fps_overlay.vert_align = align::Align::End;
        fps_overlay.position = Vec2f::new(0.0, target_win_size_y as f32);

        debug_overlay_config.font_size = 11;
        let mut trace_overlay =
            debug_ui_system.create_overlay(String_Id::from("trace"), debug_overlay_config, font);
        trace_overlay.vert_align = align::Align::Middle;
        trace_overlay.horiz_align = align::Align::Middle;
        trace_overlay.position = Vec2f::new(
            target_win_size_x as f32 * 0.5,
            target_win_size_y as f32 * 0.5,
        );
        // Trace overlay starts disabled
        debug_ui_system.set_overlay_enabled(String_Id::from("trace"), false);
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

    engine_state
        .debug_systems
        .debug_painter
        .init(gfx_resources, &engine_state.env);

    Ok(())
}

#[cfg(debug_assertions)]
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

#[cfg(not(debug_assertions))]
pub fn create_input_provider() -> Box<dyn input::provider::Input_Provider> {
    Box::new(input::default_input_provider::Default_Input_Provider::default())
}

/// Returns true if the engine should quit
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
pub fn try_create_replay_data(replay_file: &std::path::Path) -> Option<replay_data::Replay_Data> {
    match replay_data::Replay_Data::from_file(replay_file) {
        Ok(data) => Some(data),
        Err(err) => {
            eprintln!(
                "[ ERROR ] Failed to load replay data from {:?}: {}",
                replay_file, err
            );
            None
        }
    }
}

#[cfg(debug_assertions)]
pub fn maybe_update_trace_overlay(engine_state: &mut Engine_State, refresh_rate: Cfg_Var<f32>) {
    if engine_state.debug_systems.show_trace_overlay && !engine_state.time.paused {
        let t = &mut engine_state.debug_systems.trace_overlay_update_t;
        *t -= time::to_secs_frac(&engine_state.time.real_dt());

        if *t <= 0. {
            debug_update_trace_overlay(engine_state);
            engine_state.debug_systems.trace_overlay_update_t =
                refresh_rate.read(&engine_state.config);
        }
    }
}

#[cfg(debug_assertions)]
fn debug_update_trace_overlay(engine_state: &mut Engine_State) {
    use crate::core::common::colors;
    use crate::debug::overlay::Debug_Overlay;
    use crate::debug::tracer::{build_trace_trees, sort_trace_trees, Trace_Tree, Tracer_Node};

    let mut tracer = engine_state.tracer.borrow_mut();
    let overlay = engine_state
        .debug_systems
        .debug_ui_system
        .get_overlay(String_Id::from("trace"));

    overlay.clear();

    fn add_node_line(
        node: &Tracer_Node,
        total_traced_time: &Duration,
        indent: usize,
        overlay: &mut Debug_Overlay,
    ) {
        let duration = node.info.duration();
        let ratio = time::duration_ratio(&duration, total_traced_time);
        let color = colors::lerp_col(colors::GREEN, colors::RED, ratio);
        let mut line = String::new();
        for _ in 0..indent {
            line.push(' ');
        }
        line.push_str(&format!(
            "{:width$}: {:>6.3}ms ({:3}%): {:>7}",
            node.info.tag,
            node.info.duration().as_micros() as f32 * 0.001,
            (ratio * 100.0) as u32,
            node.info.n_calls,
            width = 40 - indent
        ));
        overlay.add_line_color(&line, color);
    }

    fn add_tree_lines(
        tree: &Trace_Tree,
        total_traced_time: &Duration,
        indent: usize,
        overlay: &mut Debug_Overlay,
    ) {
        add_node_line(&tree.node, total_traced_time, indent, overlay);
        for t in &tree.children {
            add_tree_lines(t, total_traced_time, indent + 1, overlay);
        }
    };

    let total_traced_time = tracer.total_traced_time();
    let traces = tracer.collate_traces();
    let mut trace_trees = build_trace_trees(traces);
    sort_trace_trees(&mut trace_trees);

    overlay.add_line_color(
        &format!(
            "{:40}: {:15}: {:7}",
            "procedure_name", "tot_time", "n_calls"
        ),
        colors::rgb(204, 0, 102),
    );
    overlay.add_line_color(&format!("{:─^60}", ""), colors::rgba(60, 60, 60, 180));
    for tree in &trace_trees {
        add_tree_lines(tree, &total_traced_time, 0, overlay);
    }
}
