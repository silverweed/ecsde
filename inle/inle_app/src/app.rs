#[cfg(debug_assertions)]
mod tracer_drawing;

use crate::app_config::App_Config;
use crate::systems::Core_Systems;
use inle_alloc::temp::Temp_Allocator;
use inle_common::units::*;
use inle_common::Maybe_Error;
use inle_core::env::Env_Info;
use inle_core::rand;
use inle_core::time;
use inle_gfx::render_window::Render_Window_Handle;
use inle_resources::gfx::Gfx_Resources;
use inle_resources::gfx::Shader_Cache;

#[cfg(debug_assertions)]
use {
    crate::systems::Debug_Systems,
    inle_cfg::Cfg_Var,
    inle_common::colors,
    inle_diagnostics::tracer,
    inle_replay::{replay_data::Replay_Data, replay_input_provider::Replay_Input_Provider},
    std::convert::TryInto,
    std::time::Duration,
};

pub struct Engine_State<'r> {
    pub should_close: bool,
    // First frame is 1
    pub cur_frame: u64,

    pub env: Env_Info,
    pub config: inle_cfg::Config,
    pub app_config: App_Config,

    pub loggers: inle_diagnostics::log::Loggers,

    pub time: time::Time,

    pub rng: rand::Default_Rng,

    pub input_state: inle_input::input_state::Input_State,
    pub systems: Core_Systems<'r>,

    pub frame_alloc: Temp_Allocator,

    pub global_batches: inle_gfx::render::batcher::Batches,

    #[cfg(debug_assertions)]
    pub debug_systems: Debug_Systems,

    #[cfg(debug_assertions)]
    pub prev_frame_time: Duration,

    #[cfg(debug_assertions)]
    pub replay_input_provider: Option<Replay_Input_Provider>,
}

pub fn create_engine_state<'r>(
    env: Env_Info,
    config: inle_cfg::Config,
    app_config: App_Config,
    loggers: inle_diagnostics::log::Loggers,
) -> Result<Engine_State<'r>, Box<dyn std::error::Error>> {
    let systems = Core_Systems::new();
    let input_state = inle_input::input_state::create_input_state(&env);
    let time = time::Time::default();
    let seed;
    #[cfg(debug_assertions)]
    {
        seed = rand::Default_Rng_Seed([
            0x12, 0x23, 0x33, 0x44, 0x44, 0xab, 0xbc, 0xcc, 0x45, 0x21, 0x72, 0x21, 0xfe, 0x31,
            0xdf, 0x46, 0xfe, 0xb4, 0x2a, 0xa9, 0x47, 0xdd, 0xd1, 0x37, 0x80, 0xfc, 0x22, 0xa1,
            0xa2, 0xb3, 0xc0, 0xfe,
        ]);
    }
    #[cfg(not(debug_assertions))]
    {
        seed = rand::new_random_seed()?;
    }
    #[cfg(debug_assertions)]
    let debug_systems = Debug_Systems::new(&config, seed);
    let rng = rand::new_rng_with_seed(seed);

    Ok(Engine_State {
        should_close: false,
        cur_frame: 0,
        env,
        config,
        app_config,
        loggers,
        time,
        rng,
        input_state,
        systems,
        global_batches: inle_gfx::render::batcher::Batches::default(),
        frame_alloc: Temp_Allocator::with_capacity(megabytes(10)),
        #[cfg(debug_assertions)]
        debug_systems,
        #[cfg(debug_assertions)]
        prev_frame_time: Duration::default(),
        #[cfg(debug_assertions)]
        replay_input_provider: None,
    })
}

#[cfg(debug_assertions)]
pub fn start_config_watch(env: &Env_Info, config: &mut inle_cfg::Config) -> Maybe_Error {
    use notify::RecursiveMode;

    let config_watcher = Box::new(inle_cfg::sync::Config_Watch_Handler::new(config));
    let config_watcher_cfg = inle_fs::file_watcher::File_Watch_Config {
        interval: Duration::from_secs(1),
        recursive_mode: RecursiveMode::Recursive,
    };
    inle_fs::file_watcher::start_file_watch(
        env.cfg_root.to_path_buf(),
        config_watcher_cfg,
        vec![config_watcher],
    )?;
    Ok(())
}

pub fn init_engine_systems(
    window: &Render_Window_Handle,
    engine_state: &mut Engine_State,
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
) -> Maybe_Error {
    gres.init();
    shader_cache.init();

    inle_input::joystick::init_joysticks(
        window,
        &engine_state.env,
        &mut engine_state.input_state.raw.joy_state,
    );
    inle_ui::init_ui(&mut engine_state.systems.ui, gres, &engine_state.env);
    engine_state.systems.long_task_mgr.start(1);

    linfo!("Number of Rayon threads: {}", rayon::current_num_threads());

    Ok(())
}

#[cfg(debug_assertions)]
pub fn start_recording(engine_state: &mut Engine_State) -> Maybe_Error {
    engine_state
        .debug_systems
        .replay_recording_system
        .start_recording_thread(&engine_state.env, &engine_state.config)
}

#[cfg(debug_assertions)]
pub fn init_engine_debug(
    engine_state: &mut Engine_State<'_>,
    gfx_resources: &mut Gfx_Resources<'_>,
    cfg: inle_debug::debug_ui::Debug_Ui_System_Config,
) -> Maybe_Error {
    use inle_common::vis_align::Align;
    use inle_debug::{graph, overlay};
    use inle_math::vector::{Vec2f, Vec2u};

    let font = gfx_resources.load_font(&inle_resources::gfx::font_path(
        &engine_state.env,
        cfg.font_name.read(&engine_state.config),
    ));

    engine_state
        .debug_systems
        .global_painter
        .init(gfx_resources, &engine_state.env);

    let (win_w, win_h) = (
        engine_state.app_config.target_win_size.0 as f32,
        engine_state.app_config.target_win_size.1 as f32,
    );

    let debug_ui = &mut engine_state.debug_systems.debug_ui;
    debug_ui.set_font(font);

    // Frame scroller
    {
        let scroller = &mut debug_ui.frame_scroller;
        scroller.size.x = (win_w * 0.75) as _;
        scroller.pos.x = (win_w * 0.125) as _;
        scroller.size.y = 35;
        scroller.pos.y = 15;
        scroller.cfg = inle_debug::frame_scroller::Debug_Frame_Scroller_Config {
            font,
            font_size: Cfg_Var::new(
                "engine/debug/frame_scroller/label_font_size",
                &engine_state.config,
            ),
            ui_scale: cfg.ui_scale,
        };
    }

    let ui_scale = cfg.ui_scale.read(&engine_state.config);
    // Debug overlays
    {
        let mut debug_overlay_config = overlay::Debug_Overlay_Config {
            ui_scale: cfg.ui_scale,
            row_spacing: Cfg_Var::new("engine/debug/overlay/row_spacing", &engine_state.config),
            font_size: cfg.font_size,
            pad_x: Cfg_Var::new("engine/debug/overlay/pad_x", &engine_state.config),
            pad_y: Cfg_Var::new("engine/debug/overlay/pad_y", &engine_state.config),
            background: Cfg_Var::new("engine/debug/overlay/background", &engine_state.config),
            font,
            ..Default::default()
        };

        let mut joy_overlay = debug_ui
            .create_overlay(sid!("joysticks"), &debug_overlay_config)
            .unwrap();
        joy_overlay.cfg.horiz_align = Align::End;
        joy_overlay.cfg.vert_align = Align::Middle;
        joy_overlay.position = v2!(win_w, win_h * 0.5);

        let time_overlay = debug_ui
            .create_overlay(sid!("time"), &debug_overlay_config)
            .unwrap();
        time_overlay.cfg.horiz_align = Align::End;
        time_overlay.cfg.vert_align = Align::End;
        time_overlay.position = Vec2f::new(win_w, win_h);

        let win_overlay = debug_ui
            .create_overlay(sid!("window"), &debug_overlay_config)
            .unwrap();
        win_overlay.cfg.horiz_align = Align::End;
        win_overlay.cfg.vert_align = Align::End;
        win_overlay.position = v2!(win_w, win_h - 20. * ui_scale);

        let fps_overlay = debug_ui
            .create_overlay(sid!("fps"), &debug_overlay_config)
            .unwrap();
        fps_overlay.cfg.vert_align = Align::End;
        fps_overlay.position = v2!(0.0, win_h);

        debug_overlay_config.fadeout_time = Cfg_Var::new(
            "engine/debug/overlay/msg/fadeout_time",
            &engine_state.config,
        );
        let msg_overlay = debug_ui
            .create_overlay(sid!("msg"), &debug_overlay_config)
            .unwrap();
        msg_overlay.cfg.horiz_align = Align::Begin;
        msg_overlay.position = Vec2f::new(0.0, 0.0);
        debug_overlay_config.fadeout_time = Cfg_Var::new_from_val(0.0);

        debug_overlay_config.pad_x =
            Cfg_Var::new("engine/debug/overlay/mouse/pad_x", &engine_state.config);
        debug_overlay_config.pad_y =
            Cfg_Var::new("engine/debug/overlay/mouse/pad_y", &engine_state.config);
        debug_overlay_config.background = Cfg_Var::new(
            "engine/debug/overlay/mouse/background",
            &engine_state.config,
        );
        let mouse_overlay = debug_ui
            .create_overlay(sid!("mouse"), &debug_overlay_config)
            .unwrap();
        mouse_overlay.cfg.horiz_align = Align::Begin;
        mouse_overlay.cfg.vert_align = Align::End;

        debug_overlay_config.background = Cfg_Var::new(
            "engine/debug/overlay/trace/background",
            &engine_state.config,
        );
        debug_overlay_config.pad_x =
            Cfg_Var::new("engine/debug/overlay/trace/pad_x", &engine_state.config);
        debug_overlay_config.pad_y =
            Cfg_Var::new("engine/debug/overlay/trace/pad_y", &engine_state.config);
        let trace_overlay = debug_ui
            .create_overlay(sid!("trace"), &debug_overlay_config)
            .unwrap();
        trace_overlay.cfg.vert_align = Align::Middle;
        trace_overlay.cfg.horiz_align = Align::Middle;
        trace_overlay.cfg.hoverable = true;
        trace_overlay.position = v2!(win_w * 0.5, win_h * 0.5);
        // Trace overlay starts disabled
        debug_ui.set_overlay_enabled(sid!("trace"), false);

        debug_overlay_config.background = Cfg_Var::new(
            "engine/debug/overlay/record/background",
            &engine_state.config,
        );
        debug_overlay_config.pad_x =
            Cfg_Var::new("engine/debug/overlay/record/pad_x", &engine_state.config);
        debug_overlay_config.pad_y =
            Cfg_Var::new("engine/debug/overlay/record/pad_y", &engine_state.config);
        debug_overlay_config.font_size = Cfg_Var::new(
            "engine/debug/overlay/record/font_size",
            &engine_state.config,
        );
        let record_overlay = debug_ui
            .create_overlay(sid!("record"), &debug_overlay_config)
            .unwrap();
        record_overlay.cfg.vert_align = Align::Begin;
        record_overlay.cfg.horiz_align = Align::Begin;
        record_overlay.position = v2!(2.0, 2.0);
    }

    // Graphs
    {
        let mut graph_config = graph::Debug_Graph_View_Config {
            grid_xstep: Some(graph::Grid_Step::Fixed_Step(5.)),
            grid_ystep: Some(graph::Grid_Step::Fixed_Step(30.)),
            ui_scale: cfg.ui_scale,
            label_font_size: Cfg_Var::new(
                "engine/debug/graphs/label_font_size",
                &engine_state.config,
            ),
            title: Some(String::from("FPS")),
            title_font_size: Cfg_Var::new(
                "engine/debug/graphs/title_font_size",
                &engine_state.config,
            ),
            color: colors::YELLOW,
            low_threshold: Some((25.0, colors::RED)),
            high_threshold: Some((55.0, colors::GREEN)),
            fixed_y_range: Some(0. ..120.),
            hoverable: true,
            show_average: true,
            font,
        };

        // FPS
        let graph = debug_ui.create_graph(sid!("fps"), &graph_config).unwrap();

        graph.size = Vec2u::new(win_w as _, (0.15 * win_h) as _);

        // Prev frame time before display
        graph_config.show_average = true;
        graph_config.grid_ystep = Some(graph::Grid_Step::Fixed_Subdivs(4));
        graph_config.fixed_y_range = None;
        graph_config.title = Some(String::from("PrevFrameTime"));
        graph_config.low_threshold = Some((17., colors::GREEN));
        graph_config.high_threshold = Some((34., colors::RED));
        let graph = debug_ui
            .create_graph(sid!("prev_frame_time"), &graph_config)
            .unwrap();
        graph.pos.y = (0.15 * win_h) as u32;
        graph.size = Vec2u::new(win_w as _, (0.15 * win_h) as _);

        // Function profile
        graph_config.fixed_y_range = None;
        graph_config.grid_ystep = Some(graph::Grid_Step::Fixed_Subdivs(4));
        graph_config.low_threshold = Some((0.01, colors::GREEN));
        graph_config.high_threshold = Some((10., colors::RED));
        graph_config.title = None;
        graph_config.hoverable = true;
        let graph = debug_ui
            .create_graph(sid!("fn_profile"), &graph_config)
            .unwrap();
        graph.pos.y = (0.3 * win_h) as u32;
        graph.size = Vec2u::new(win_w as _, (0.15 * win_h) as _);
    }

    {
        let log_window_config = inle_debug::log_window::Log_Window_Config {
            font,
            font_size: Cfg_Var::new("engine/debug/log_window/font_size", &engine_state.config),
            title: std::borrow::Cow::Borrowed("Log"),
            title_font_size: Cfg_Var::new(
                "engine/debug/log_window/title_font_size",
                &engine_state.config,
            ),
            header_height: Cfg_Var::new(
                "engine/debug/log_window/header_height",
                &engine_state.config,
            ),
            ui_scale: cfg.ui_scale,
            pad_x: Cfg_Var::new("engine/debug/log_window/pad_x", &engine_state.config),
            pad_y: Cfg_Var::new("engine/debug/log_window/pad_y", &engine_state.config),
            linesep: Cfg_Var::new("engine/debug/log_window/linesep", &engine_state.config),
            scrolled_lines: Cfg_Var::new(
                "engine/debug/log_window/scrolled_lines",
                &engine_state.config,
            ),
            page_scrolled_lines: Cfg_Var::new(
                "engine/debug/log_window/page_scrolled_lines",
                &engine_state.config,
            ),
            max_lines: 2000,
        };
        let log_window = debug_ui
            .create_log_window(sid!("log_window"), &log_window_config)
            .unwrap();
        let logger = log_window.create_logger();
        log_window.size = Vec2u::new((win_w * 0.8) as _, (win_h * 0.8) as _);
        log_window.pos.y += (win_h as u32 - log_window.size.y) / 2;
        log_window.pos.x += (win_w as u32 - log_window.size.x) / 2;
        debug_ui.set_log_window_enabled(sid!("log_window"), true);

        inle_diagnostics::log::add_logger(&mut engine_state.loggers, logger);
    }

    debug_ui.cfg = cfg;

    {
        use inle_input::bindings::{Input_Action, Input_Action_Simple};

        let console = &mut engine_state.debug_systems.console.lock().unwrap();
        console.size = Vec2u::new(win_w as _, win_h as u32 / 2);
        console.toggle_console_keys = engine_state
            .input_state
            .bindings
            .get_all_actions_triggering(sid!("toggle_console"))
            .iter()
            .filter_map(|action| {
                if let Input_Action {
                    action: Input_Action_Simple::Key(key),
                    .. // @Robustness: right now, the console only supports an unmodified key to close.
                } = action
                {
                    Some(*key)
                } else {
                    None
                }
            })
            .collect();
        console.init(inle_debug::console::Console_Config {
            font,
            font_size: Cfg_Var::new("engine/debug/console/font_size", &engine_state.config),
            pad_x: Cfg_Var::new("engine/debug/console/pad_x", &engine_state.config),
            linesep: Cfg_Var::new("engine/debug/console/linesep", &engine_state.config),
            opacity: Cfg_Var::new("engine/debug/console/opacity", &engine_state.config),
            cur_line_opacity: Cfg_Var::new(
                "engine/debug/console/cur_line_opacity",
                &engine_state.config,
            ),
            ui_scale: debug_ui.cfg.ui_scale,
        });
    }

    Ok(())
}

/// Returns true if the engine should quit
pub fn handle_core_actions(
    actions: &[inle_input::core_actions::Core_Action],
    window: &mut Render_Window_Handle,
    engine_state: &mut Engine_State,
) -> bool {
    use inle_input::core_actions::Core_Action;

    for action in actions.iter() {
        match action {
            Core_Action::Quit => return true,
            Core_Action::Resize(new_width, new_height) => {
                inle_gfx::render_window::resize_keep_ratio(window, *new_width, *new_height);
            }
            Core_Action::Focus_Lost => {
                engine_state.input_state.raw.kb_state.modifiers_pressed = 0;
                inle_input::mouse::reset_mouse_state(&mut engine_state.input_state.raw.mouse_state);
            }
            _ => unimplemented!(),
        }
    }

    false
}

#[cfg(debug_assertions)]
pub fn try_create_replay_data(replay_file: &std::path::Path) -> Option<Replay_Data> {
    match Replay_Data::from_file(replay_file) {
        Ok(data) => Some(data),
        Err(err) => {
            lerr!("Failed to load replay data from {:?}: {}", replay_file, err);
            None
        }
    }
}

#[cfg(debug_assertions)]
pub fn set_replay_data(engine_state: &mut Engine_State, replay_data: Replay_Data) {
    engine_state.replay_input_provider = Some(Replay_Input_Provider::new(replay_data));
}

#[cfg(debug_assertions)]
pub fn update_traces(engine_state: &mut Engine_State, refresh_rate: Cfg_Var<f32>) {
    use crate::systems::Overlay_Shown;
    use inle_diagnostics::{prelude, tracer::Tracer_Node};
    use std::thread::ThreadId;

    // @Speed: do a pass on this

    let traces: Vec<(ThreadId, Vec<Tracer_Node>)> = {
        // Note: we unlock the tracer asap to prevent deadlocks.
        // We're not keeping any reference to it anyway.
        let mut tracers = prelude::DEBUG_TRACERS.lock().unwrap();
        tracers
            .iter_mut()
            .map(|(&thread_id, tracer)| {
                let mut tracer = tracer.lock().unwrap();
                let traces = std::mem::take(&mut tracer.saved_traces);
                (thread_id, traces)
            })
            .collect()
    };

    // Merge the individual threads' traces into a single array. This is used to generate the unified profile view.
    // If necessary we also collect all tree root nodes that will be used by the thread view
    // Vec<(tid, Vec<Node>)> => Vec<Node>
    let is_threads_view = matches!(
        engine_state.debug_systems.show_overlay,
        Overlay_Shown::Threads
    );
    let mut merged_traces = vec![];
    let mut trace_roots = vec![];
    let mut patch_offset = 0;
    for (tid, nodes) in traces {
        let nodes_len = nodes.len();
        merged_traces.reserve(nodes_len);
        for mut node in nodes {
            // When we merge the different threads' traces we must fix the parent idx of the nodes,
            // otherwise we'll have several nodes having the same parent_idx because every thread
            // has its own local tree and the merged traces will be all messed up.
            if let Some(idx) = node.parent_idx {
                node.parent_idx = Some(idx + patch_offset);
            } else if is_threads_view {
                trace_roots.push((tid, node.clone()));
            }
            merged_traces.push(node);
        }
        patch_offset += nodes_len;
    }

    let final_traces = tracer::collate_traces(&merged_traces);

    let debug_log = &mut engine_state.debug_systems.log;
    let scroller = &engine_state.debug_systems.debug_ui.frame_scroller;
    if !scroller.manually_selected {
        debug_log.push_trace(&final_traces);
    }
    let trace_realtime = Cfg_Var::<bool>::new("engine/debug/trace/realtime", &engine_state.config)
        .read(&engine_state.config);

    match engine_state.debug_systems.show_overlay {
        Overlay_Shown::Trace => {
            let t = &mut engine_state.debug_systems.trace_overlay_update_t;
            if trace_realtime || !engine_state.time.paused {
                *t -= engine_state.time.real_dt().as_secs_f32();
            }

            if *t <= 0. {
                let trace_view_flat =
                    Cfg_Var::<bool>::new("engine/debug/trace/view_flat", &engine_state.config)
                        .read(&engine_state.config);
                if trace_view_flat {
                    tracer_drawing::update_trace_flat_overlay(engine_state);
                } else {
                    tracer_drawing::update_trace_tree_overlay(engine_state);
                }
                engine_state.debug_systems.trace_overlay_update_t =
                    refresh_rate.read(&engine_state.config);

                if !trace_realtime && engine_state.time.paused {
                    // Don't bother refreshing this the next frame: we're paused.
                    engine_state.debug_systems.trace_overlay_update_t = 0.1;
                }
            }
        }

        Overlay_Shown::Threads => {
            tracer_drawing::update_thread_overlay(engine_state, &trace_roots);
        }

        _ => {}
    }

    // Function trace graph
    if trace_realtime || !engine_state.time.paused {
        let sid_trace = sid!("trace");
        let debug_systems = &mut engine_state.debug_systems;
        let trace_hover_data = debug_systems
            .debug_ui
            .get_overlay(sid_trace)
            .hover_data
            .clone();
        if trace_hover_data.just_selected {
            if let Some(tracer_selected_idx) = trace_hover_data.selected_line {
                let fn_name: String = debug_systems.debug_ui.get_overlay(sid_trace).lines
                    [tracer_selected_idx]
                    .metadata
                    .get(&sid!("full_tag"))
                    .map(|x| x.clone().try_into().ok())
                    .flatten()
                    .unwrap_or_default();
                set_traced_fn(debug_systems, fn_name);
            } else {
                set_traced_fn(debug_systems, String::default());
            }
        }

        if !debug_systems.traced_fn.is_empty() {
            debug_systems
                .debug_ui
                .set_graph_enabled(sid!("fn_profile"), true);

            let graph = debug_systems.debug_ui.get_graph(sid!("fn_profile"));

            let flattened_traces = tracer::flatten_traces(&final_traces);
            tracer_drawing::update_graph_traced_fn(
                flattened_traces,
                graph,
                if trace_realtime {
                    engine_state.time.real_time()
                } else {
                    engine_state.time.game_time()
                },
                &debug_systems.traced_fn,
                engine_state.cur_frame,
            );
        } else {
            debug_systems
                .debug_ui
                .set_graph_enabled(sid!("fn_profile"), false);
        }
    }
}

#[cfg(debug_assertions)]
pub fn set_traced_fn(debug_systems: &mut Debug_Systems, fn_name: String) {
    debug_systems.traced_fn = fn_name.clone();
    let graph = debug_systems.debug_ui.get_graph(sid!("fn_profile"));
    graph.cfg.title = Some(fn_name);
    graph.data.points.clear();
    graph.selected_point = None;
}
