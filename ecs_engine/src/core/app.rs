use super::app_config::App_Config;
use super::env::Env_Info;
use super::time;
use crate::alloc::temp::Temp_Allocator;
use crate::cfg;
use crate::common::units::*;
use crate::common::Maybe_Error;
use crate::core::rand;
use crate::core::systems::Core_Systems;
use crate::gfx;
use crate::input;

#[cfg(debug_assertions)]
use {
    crate::cfg::Cfg_Var,
    crate::common::stringid::String_Id,
    crate::core::systems::Debug_Systems,
    crate::debug,
    crate::fs,
    crate::replay::{replay_data, replay_input_provider},
    crate::resources::{self, gfx::Gfx_Resources},
    std::time::Duration,
};

pub struct Engine_State<'r> {
    pub should_close: bool,
    // First frame is 1
    pub cur_frame: u64,

    pub env: Env_Info,
    pub config: cfg::Config,
    pub app_config: App_Config,

    pub time: time::Time,

    pub rng: rand::Default_Rng,

    pub input_state: input::input_system::Input_State,
    pub systems: Core_Systems<'r>,

    pub frame_alloc: Temp_Allocator,

    pub global_batches: gfx::render::batcher::Batches,

    #[cfg(debug_assertions)]
    pub debug_systems: Debug_Systems,

    #[cfg(debug_assertions)]
    pub prev_frame_time: Duration,

    #[cfg(debug_assertions)]
    pub replay_data: Option<replay_data::Replay_Data>,
}

pub fn create_engine_state<'r>(
    env: Env_Info,
    config: cfg::Config,
    app_config: App_Config,
) -> Result<Engine_State<'r>, Box<dyn std::error::Error>> {
    let systems = Core_Systems::new();
    let input_state = input::input_system::create_input_state(&env);
    let time = time::Time::default();
    #[cfg(debug_assertions)]
    let debug_systems = Debug_Systems::new(&config);
    let rng;
    #[cfg(debug_assertions)]
    {
        rng = rand::new_rng_with_seed([
            0x12, 0x23, 0x33, 0x44, 0x44, 0xab, 0xbc, 0xcc, 0x45, 0x21, 0x72, 0x21, 0xfe, 0x31,
            0xdf, 0x46, 0xfe, 0xb4, 0x2a, 0xa9, 0x47, 0xdd, 0xd1, 0x37, 0x80, 0xfc, 0x22, 0xa1,
            0xa2, 0xb3, 0xc0, 0xfe,
        ])?;
    }
    #[cfg(not(debug_assertions))]
    {
        rng = rand::new_rng_with_random_seed()?;
    }

    Ok(Engine_State {
        should_close: false,
        cur_frame: 0,
        env,
        config,
        app_config,
        time,
        rng,
        input_state,
        systems,
        global_batches: gfx::render::batcher::Batches::default(),
        frame_alloc: Temp_Allocator::with_capacity(megabytes(1)),
        #[cfg(debug_assertions)]
        debug_systems,
        #[cfg(debug_assertions)]
        prev_frame_time: Duration::default(),
        #[cfg(debug_assertions)]
        replay_data: None,
    })
}

#[cfg(debug_assertions)]
pub fn start_config_watch(env: &Env_Info, config: &mut cfg::Config) -> Maybe_Error {
    use notify::RecursiveMode;

    let config_watcher = Box::new(cfg::sync::Config_Watch_Handler::new(config));
    let config_watcher_cfg = fs::file_watcher::File_Watch_Config {
        interval: Duration::from_secs(1),
        recursive_mode: RecursiveMode::Recursive,
    };
    fs::file_watcher::start_file_watch(
        env.cfg_root.to_path_buf(),
        config_watcher_cfg,
        vec![config_watcher],
    )?;
    Ok(())
}

pub fn init_engine_systems(engine_state: &mut Engine_State) -> Maybe_Error {
    input::joystick_state::init_joysticks(&mut engine_state.input_state.joy_state);

    linfo!("Number of Rayon threads: {}", rayon::current_num_threads());

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
    cfg: debug::debug_ui::Debug_Ui_System_Config,
) -> Maybe_Error {
    use crate::common::colors;
    use crate::common::vector::{Vec2f, Vec2u};
    use crate::gfx::align::Align;
    use debug::{fadeout_overlay, graph, overlay};

    const FONT: &str = "Hack-Regular.ttf";

    let font = gfx_resources.load_font(&resources::gfx::font_path(&engine_state.env, FONT));

    engine_state
        .debug_systems
        .global_painter()
        .init(gfx_resources, &engine_state.env);

    // @Robustness: add font validity check

    let (win_w, win_h) = (
        engine_state.app_config.target_win_size.0 as f32,
        engine_state.app_config.target_win_size.1 as f32,
    );
    let ui_scale = cfg.ui_scale;
    let debug_ui = &mut engine_state.debug_systems.debug_ui;
    debug_ui.cfg = cfg;

    // Frame scroller
    {
        let scroller = &mut debug_ui.frame_scroller;
        scroller.size.x = (win_w * 0.75) as _;
        scroller.pos.x = (win_w * 0.125) as _;
        scroller.size.y = 35;
        scroller.pos.y = 15;
        scroller.cfg = debug::frame_scroller::Debug_Frame_Scroller_Config {
            font,
            font_size: (7. * ui_scale) as _,
        };
    }

    // Debug overlays
    {
        let mut debug_overlay_config = overlay::Debug_Overlay_Config {
            row_spacing: 2.0 * ui_scale,
            font_size: (10.0 * ui_scale) as _,
            pad_x: 5.0 * ui_scale,
            pad_y: 5.0 * ui_scale,
            background: colors::rgba(25, 25, 25, 210),
            font,
            ..Default::default()
        };

        let mut joy_overlay = debug_ui
            .create_overlay(String_Id::from("joysticks"), debug_overlay_config)
            .unwrap();
        joy_overlay.config.horiz_align = Align::End;
        joy_overlay.config.vert_align = Align::Middle;
        joy_overlay.position = Vec2f::new(win_w, win_h * 0.5);

        debug_overlay_config.font_size = (13.0 * ui_scale) as _;
        let time_overlay = debug_ui
            .create_overlay(String_Id::from("time"), debug_overlay_config)
            .unwrap();
        time_overlay.config.horiz_align = Align::End;
        time_overlay.config.vert_align = Align::End;
        time_overlay.position = Vec2f::new(win_w, win_h);

        let win_overlay = debug_ui
            .create_overlay(String_Id::from("window"), debug_overlay_config)
            .unwrap();
        win_overlay.config.horiz_align = Align::End;
        win_overlay.config.vert_align = Align::End;
        win_overlay.position = Vec2f::new(win_w, win_h - 20. * ui_scale);

        let fps_overlay = debug_ui
            .create_overlay(String_Id::from("fps"), debug_overlay_config)
            .unwrap();
        fps_overlay.config.vert_align = Align::End;
        fps_overlay.position = Vec2f::new(0.0, win_h);

        debug_overlay_config.pad_x = 0.;
        debug_overlay_config.pad_y = 0.;
        debug_overlay_config.background = colors::TRANSPARENT;
        let mouse_overlay = debug_ui
            .create_overlay(String_Id::from("mouse"), debug_overlay_config)
            .unwrap();
        mouse_overlay.config.horiz_align = Align::Begin;
        mouse_overlay.config.vert_align = Align::Begin;

        debug_overlay_config.background = colors::rgba(20, 20, 20, 220);
        let trace_overlay = debug_ui
            .create_overlay(String_Id::from("trace"), debug_overlay_config)
            .unwrap();
        trace_overlay.config.vert_align = Align::Middle;
        trace_overlay.config.horiz_align = Align::Middle;
        trace_overlay.position = Vec2f::new(win_w * 0.5, win_h * 0.5);
        // Trace overlay starts disabled
        debug_ui.set_overlay_enabled(String_Id::from("trace"), false);
    }

    // Debug fadeout overlays
    {
        let fadeout_overlay_config = fadeout_overlay::Fadeout_Debug_Overlay_Config {
            row_spacing: 2.0 * ui_scale,
            font_size: (20.0 * ui_scale) as _,
            pad_x: 5.0 * ui_scale,
            pad_y: 5.0 * ui_scale,
            background: colors::rgba(25, 25, 25, 210),
            fadeout_time: Duration::from_secs(3),
            max_rows: (30.0 / ui_scale.max(0.1)) as _,
            font,
            ..Default::default()
        };

        let fadeout_overlay = debug_ui
            .create_fadeout_overlay(String_Id::from("msg"), fadeout_overlay_config)
            .unwrap();
        fadeout_overlay.config.horiz_align = Align::Begin;
        fadeout_overlay.position = Vec2f::new(0.0, 0.0);
    }

    // Graphs
    {
        let mut graph_config = graph::Debug_Graph_View_Config {
            grid_xstep: Some(graph::Grid_Step::Fixed_Step(5.)),
            grid_ystep: Some(graph::Grid_Step::Fixed_Step(30.)),
            label_font_size: (10.0 * ui_scale) as _,
            title: Some(String::from("FPS")),
            title_font_size: (18.0 * ui_scale) as _,
            color: colors::YELLOW,
            low_threshold: Some((25.0, colors::RED)),
            high_threshold: Some((55.0, colors::GREEN)),
            fixed_y_range: Some(0. ..120.),
            font,
        };

        // FPS
        let graph = engine_state
            .debug_systems
            .debug_ui
            .create_graph(String_Id::from("fps"), graph_config.clone())
            .unwrap();

        graph.size = Vec2u::new(win_w as _, (0.15 * win_h) as _);

        // Prev frame time before display
        graph_config.grid_ystep = Some(graph::Grid_Step::Fixed_Subdivs(4));
        graph_config.title = Some(String::from("PrevFrameTime"));
        graph_config.low_threshold = Some((17., colors::GREEN));
        graph_config.high_threshold = Some((34., colors::RED));
        let graph = engine_state
            .debug_systems
            .debug_ui
            .create_graph(String_Id::from("prev_frame_time"), graph_config)
            .unwrap();
        graph.pos.y = (0.15 * win_h) as u32;
        graph.size = Vec2u::new(win_w as _, (0.15 * win_h) as _);
    }

    {
        use crate::input::bindings::Input_Action;

        let console = &mut engine_state.debug_systems.console;
        console.size = Vec2u::new(win_w as _, win_h as u32 / 2);
        console.font_size = (console.font_size as f32 * ui_scale) as _;
        console.toggle_console_keys = engine_state
            .input_state
            .bindings
            .get_all_actions_triggering(String_Id::from("toggle_console"))
            .iter()
            .filter_map(|action| {
                if let Input_Action::Key(key) = action {
                    Some(*key)
                } else {
                    None
                }
            })
            .collect();
        console.init(gfx_resources, &engine_state.env);
    }

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
            lerr!("Failed to load replay data from {:?}: {}", replay_file, err);
            None
        }
    }
}

#[cfg(debug_assertions)]
pub fn update_traces(engine_state: &mut Engine_State, refresh_rate: Cfg_Var<f32>) {
    use crate::debug::tracer;
    use crate::prelude;

    let mut tracer = prelude::DEBUG_TRACER.lock().unwrap();

    let debug_log = &mut engine_state.debug_systems.log;
    let traces = tracer.saved_traces.split_off(0);
    let final_traces = tracer::collate_traces(&traces);

    let scroller = &engine_state.debug_systems.debug_ui.frame_scroller;
    if !scroller.manually_selected {
        debug_log.push_trace(&final_traces);
    }

    if engine_state.debug_systems.show_trace_overlay {
        let t = &mut engine_state.debug_systems.trace_overlay_update_t;
        if !engine_state.time.paused {
            *t -= engine_state.time.real_dt().as_secs_f32();
        }

        if *t <= 0. {
            update_trace_overlay(engine_state);
            engine_state.debug_systems.trace_overlay_update_t =
                refresh_rate.read(&engine_state.config);

            if engine_state.time.paused {
                // Don't bother refreshing this the next frame: we're paused.
                engine_state.debug_systems.trace_overlay_update_t = 0.1;
            }
        }
    }
}

#[cfg(debug_assertions)]
fn update_trace_overlay(engine_state: &mut Engine_State) {
    use crate::common::colors;
    use crate::debug::overlay::Debug_Overlay;
    use crate::debug::tracer::{self, Trace_Tree, Tracer_Node_Final};

    fn add_node_line(
        node: &Tracer_Node_Final,
        total_traced_time: &Duration,
        indent: usize,
        overlay: &mut Debug_Overlay,
    ) {
        let duration = node.info.tot_duration();
        let n_calls = node.info.n_calls();
        let ratio = time::duration_ratio(&duration, total_traced_time);
        let color = colors::lerp_col(colors::GREEN, colors::RED, ratio);
        let mut line = String::new();
        for _ in 0..indent {
            line.push(' ');
        }
        let duration_ms = time::to_ms_frac(&duration);
        line.push_str(&format!(
            "{:width$}: {:>6.3}ms ({:3}%): {:>7}: {:6.3}ms",
            node.info.tag,
            duration_ms,
            (ratio * 100.0) as u32,
            n_calls,
            duration_ms / n_calls as f32,
            width = 40 - indent
        ));
        let bg_col = colors::Color { a: 50, ..color };
        overlay.add_line_color_with_bg_fill(&line, color, (bg_col, ratio));
    }

    fn add_tree_lines(
        tree: &Trace_Tree,
        total_traced_time: &Duration,
        indent: usize,
        overlay: &mut Debug_Overlay,
        prune_duration: &Duration,
    ) {
        if tree.node.info.tot_duration() < *prune_duration {
            return;
        }

        add_node_line(&tree.node, total_traced_time, indent, overlay);
        for t in &tree.children {
            add_tree_lines(t, total_traced_time, indent + 1, overlay, prune_duration);
        }
    };

    let ui_scale = engine_state.debug_systems.debug_ui.cfg.ui_scale;
    let scroller = &engine_state.debug_systems.debug_ui.frame_scroller;
    let debug_log = &mut engine_state.debug_systems.log;
    let frame = scroller.get_real_selected_frame();
    let traces = &debug_log.get_frame(frame).unwrap().traces;
    let overlay = engine_state
        .debug_systems
        .debug_ui
        .get_overlay(String_Id::from("trace"));

    overlay.clear();

    let total_traced_time = tracer::total_traced_time(traces);
    let mut trace_trees = tracer::build_trace_trees(traces);
    tracer::sort_trace_trees(&mut trace_trees);

    let font_size = cfg::Cfg_Var::<i32>::new("engine/debug/trace/font_size", &engine_state.config)
        .read(&engine_state.config);
    overlay.config.font_size = (font_size as f32 * ui_scale) as _;

    let prune_duration_ms =
        cfg::Cfg_Var::<f32>::new("engine/debug/trace/prune_duration_ms", &engine_state.config)
            .read(&engine_state.config);
    let prune_duration = Duration::from_secs_f32(prune_duration_ms * 0.001);

    overlay.add_line_color(
        &format!(
            "frame {} | debug_log_mem {} | temp_mem_max_usage {} / {}",
            frame,
            format_bytes_pretty(debug_log.mem_used),
            format_bytes_pretty(engine_state.frame_alloc.high_water_mark),
            format_bytes_pretty(engine_state.frame_alloc.cap)
        ),
        colors::rgb(144, 144, 144),
    );
    overlay.add_line_color(
        &format!(
            "{:<39}: {:<15}: {:7}: {:>7}",
            "procedure_name", "tot_time", "n_calls", "t/call"
        ),
        colors::rgb(204, 0, 102),
    );
    overlay.add_line_color(&format!("{:─^80}", ""), colors::rgba(60, 60, 60, 180));
    for tree in &trace_trees {
        add_tree_lines(tree, &total_traced_time, 0, overlay, &prune_duration);
    }
}
