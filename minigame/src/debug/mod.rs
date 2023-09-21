mod console_executor;

use super::{Game_Resources, Game_State};
use inle_app::debug_systems::Debug_Systems;
use inle_cfg::Cfg_Var;
use inle_debug::console::Console_Status;
use inle_input::input_state::Action_Kind;
use std::convert::TryInto;

pub fn init_debug(game_state: &mut Game_State, game_res: &mut Game_Resources) {
    let ui_scale = Cfg_Var::<f32>::new("engine/debug/ui/ui_scale", &game_state.config);
    let font_name = Cfg_Var::<String>::new("engine/debug/ui/font", &game_state.config);
    let cfg = inle_debug::debug_ui::Debug_Ui_System_Config {
        target_win_size: game_state.app_config.target_win_size,
        ui_scale,
        font_name,
        font_size: Cfg_Var::new("engine/debug/ui/font_size", &game_state.config),
    };

    inle_app::app::init_engine_debug(
        &game_state.env,
        &game_state.config,
        &mut game_state.debug_systems,
        &game_state.app_config,
        &game_state.input,
        &mut game_res.gfx,
        cfg,
    )
    .unwrap();

    init_game_debug(game_state, game_res);
}

fn init_game_debug(game_state: &mut Game_State, game_res: &mut Game_Resources) {
    use inle_common::vis_align::Align;
    use inle_debug::overlay::Debug_Overlay_Config;
    use inle_math::vector::Vec2f;

    let debug_ui = &mut game_state.debug_systems.debug_ui;
    let cfg = &game_state.config;

    let font_name = debug_ui.cfg.font_name.read(cfg);
    let font = game_res
        .gfx
        .load_font(&inle_resources::gfx::font_path(&game_state.env, font_name));

    let (win_w, win_h) = game_state.app_config.target_win_size;

    {
        // Frame scroller
        let scroller = &mut debug_ui.frame_scroller;
        let fps = 60;
        let log_len = game_state.debug_systems.log.max_hist_len;
        scroller.n_frames = fps as _;
        scroller.n_seconds = (log_len / fps as u32) as _;
    }

    let overlay_cfg = Debug_Overlay_Config {
        row_spacing: Cfg_Var::new("debug/overlay/gameplay/row_spacing", cfg),
        font_size: debug_ui.cfg.font_size,
        pad_x: Cfg_Var::new("debug/overlay/gameplay/pad_x", cfg),
        pad_y: Cfg_Var::new("debug/overlay/gameplay/pad_y", cfg),
        background: Cfg_Var::new("debug/overlay/gameplay/background", cfg),
        ui_scale: debug_ui.cfg.ui_scale,
        font,
        ..Default::default()
    };
    // Entities overlay
    let ui_scale = debug_ui.cfg.ui_scale.read(cfg);
    let overlay = debug_ui
        .create_overlay(sid!("entities"), &overlay_cfg)
        .unwrap();
    overlay.cfg.vert_align = Align::End;
    overlay.cfg.horiz_align = Align::Begin;
    overlay.position = Vec2f::new(0.0, win_h as f32 - 24. * ui_scale);

    // Camera overlay
    let overlay = debug_ui
        .create_overlay(sid!("camera"), &overlay_cfg)
        .unwrap();
    overlay.cfg.vert_align = Align::End;
    overlay.cfg.horiz_align = Align::End;
    overlay.position = Vec2f::new(win_w as f32, win_h as f32 - 40. * ui_scale);

    // Physics overlay
    let overlay = debug_ui
        .create_overlay(sid!("physics"), &overlay_cfg)
        .unwrap();
    overlay.cfg.vert_align = Align::End;
    overlay.cfg.horiz_align = Align::Begin;
    overlay.position = Vec2f::new(0.0, win_h as f32 - 46. * ui_scale);

    // Console hints
    let console = &mut game_state.debug_systems.console.lock().unwrap();
    console.add_hints(
        "",
        crate::debug::console_executor::ALL_CMD_STRINGS
            .iter()
            .map(|s| String::from(*s)),
    );
    console.add_hints("var", game_state.config.get_all_pairs().map(|(k, _)| k));
    console.add_hints(
        "toggle",
        game_state.config.get_all_pairs().filter_map(|(k, v)| {
            if let inle_cfg::Cfg_Value::Bool(_) = v {
                Some(k)
            } else {
                None
            }
        }),
    );
}

pub fn start_debug_frame(
    debug_systems: &mut Debug_Systems,
    time: &inle_core::time::Time,
    cur_frame: u64,
) {
    inle_diagnostics::prelude::DEBUG_TRACERS
        .lock()
        .unwrap()
        .values_mut()
        .for_each(|t| t.lock().unwrap().start_frame());

    let log = &mut debug_systems.log;

    if !time.paused {
        if time.was_paused() {
            // Just resumed
            debug_systems.debug_ui.frame_scroller.manually_selected = false;
            log.reset_from_frame(cur_frame);
        }
        log.start_frame();
    }
}

pub fn update_debug(game_state: &mut Game_State, game_res: &mut Game_Resources) {
    trace!("update_debug");

    update_console(game_state);
    update_scroller(game_state);

    handle_debug_actions(game_state, game_res);
}

fn update_console(game_state: &mut Game_State) {
    trace!("console::update");

    let mut console = game_state.debug_systems.console.lock().unwrap();
    let mut output = vec![];
    let mut commands = vec![];
    if console.status == Console_Status::Open {
        console.update(&game_state.input);

        while let Some(cmd) = console.pop_enqueued_cmd() {
            if !cmd.is_empty() {
                commands.push(cmd);
            }
        }

        if !commands.is_empty() {
            drop(console);

            for cmd in commands {
                let maybe_output = console_executor::execute(&cmd, game_state);
                if let Some(out) = maybe_output {
                    output.push(out);
                }
            }

            console = game_state.debug_systems.console.lock().unwrap();
        }

        for (out, color) in output {
            console.output_line(format!(">> {}", out), color);
        }
    }

    let actions = &game_state.input.processed.game_actions;
    if actions.contains(&(sid!("toggle_console"), Action_Kind::Pressed)) {
        console.toggle();
    }

    inle_win::window::set_key_repeat_enabled(
        &mut game_state.window,
        console.status == Console_Status::Open,
    );
}

fn update_scroller(game_state: &mut Game_State) {
    let scroller = &mut game_state.debug_systems.debug_ui.frame_scroller;
    let prev_selected_frame = scroller.cur_frame;
    let prev_selected_second = scroller.cur_second;
    let was_manually_selected = scroller.manually_selected;

    scroller.handle_events(&game_state.input.raw.events);

    if scroller.cur_frame != prev_selected_frame
        || scroller.cur_second != prev_selected_second
        || was_manually_selected != scroller.manually_selected
    {
        game_state.time.paused = scroller.manually_selected;
        game_state.debug_systems.trace_overlay_update_t = 0.;
    }
}

pub fn update_traces(
    refresh_rate: Cfg_Var<f32>,
    debug_systems: &mut Debug_Systems,
    config: &inle_cfg::config::Config,
    time: &inle_core::time::Time,
    cur_frame: u64,
    frame_alloc: &mut inle_alloc::temp::Temp_Allocator,
) {
    use inle_app::app::tracer_drawing;
    use inle_app::debug_systems::Overlay_Shown;
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
    let is_threads_view = matches!(debug_systems.show_overlay, Overlay_Shown::Threads);
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

    let final_traces = inle_diagnostics::tracer::collate_traces(&merged_traces);

    let debug_log = &mut debug_systems.log;
    let scroller = &debug_systems.debug_ui.frame_scroller;
    if !scroller.manually_selected {
        debug_log.push_trace(&final_traces);
    }
    let trace_realtime = Cfg_Var::<bool>::new("engine/debug/trace/realtime", &config).read(&config);

    match debug_systems.show_overlay {
        Overlay_Shown::Trace => {
            let t = &mut debug_systems.trace_overlay_update_t;
            if trace_realtime || !time.paused {
                *t -= time.real_dt().as_secs_f32();
            }

            if *t <= 0. {
                let trace_view_flat =
                    Cfg_Var::<bool>::new("engine/debug/trace/view_flat", &config).read(&config);
                if trace_view_flat {
                    tracer_drawing::update_trace_flat_overlay(debug_systems, config, frame_alloc);
                } else {
                    tracer_drawing::update_trace_tree_overlay(debug_systems, config, frame_alloc);
                }
                debug_systems.trace_overlay_update_t = refresh_rate.read(&config);

                if !trace_realtime && time.paused {
                    // Don't bother refreshing this the next frame: we're paused.
                    debug_systems.trace_overlay_update_t = 0.1;
                }
            }
        }

        Overlay_Shown::Threads => {
            //tracer_drawing::update_thread_overlay(debug_systems, app_config, &trace_roots);
        }

        _ => {}
    }

    // Function trace graph
    if trace_realtime || !time.paused {
        let sid_trace = sid!("trace");
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

            let flattened_traces = inle_diagnostics::tracer::flatten_traces(&final_traces);
            tracer_drawing::update_graph_traced_fn(
                flattened_traces,
                graph,
                if trace_realtime {
                    time.real_time()
                } else {
                    time.game_time()
                },
                &debug_systems.traced_fn,
                cur_frame,
            );
        } else {
            debug_systems
                .debug_ui
                .set_graph_enabled(sid!("fn_profile"), false);
        }
    }
}

pub fn render_debug<'a, 'r>(
    debug_systems: &mut Debug_Systems,
    window: &mut inle_gfx::render_window::Render_Window_Handle,
    input: &inle_input::input_state::Input_State,
    config: &inle_cfg::Config,
    temp_alloc: &mut inle_alloc::temp::Temp_Allocator,
    time: &mut inle_core::time::Time,
    gres: &'a mut inle_resources::gfx::Gfx_Resources<'r>,
) {
    use inle_math::transform::Transform2D;

    let real_dt = time.real_dt();

    // Draw debug calipers
    {
        let calipers = &debug_systems.calipers;
        // @Incomplete: use level camera transform
        let camera_xform = Transform2D::default();
        calipers.draw(
            window,
            &mut debug_systems.global_painter,
            &camera_xform,
            input,
        );
    }

    // Draw global debug painter
    let painter = &mut debug_systems.global_painter;
    painter.draw(window, gres, &Transform2D::default());
    painter.clear();

    // Draw debug UI
    {
        let debug_ui = &mut debug_systems.debug_ui;
        let prev_selected = debug_ui.get_graph(sid!("fn_profile")).get_selected_point();

        debug_ui.update_and_draw(
            &real_dt,
            window,
            gres,
            input,
            &debug_systems.log,
            config,
            temp_alloc,
        );

        let profile_graph = debug_ui.get_graph(sid!("fn_profile"));
        let cur_selected = profile_graph.get_selected_point();
        if cur_selected != prev_selected {
            time.paused = cur_selected.is_some();
            debug_ui.frame_scroller.manually_selected = cur_selected.is_some();
            if let Some(sel) = cur_selected {
                let profile_graph = debug_ui.get_graph(sid!("fn_profile"));
                // @Robustness @Refactoring: this should be a u64
                let real_frame: u32 = profile_graph
                    .data
                    .get_point_metadata(sel.index, sid!("real_frame"))
                    .expect("Failed to get point frame metadata!");
                debug_ui
                    .frame_scroller
                    .set_real_selected_frame(real_frame as u64);
            }
        }
    }

    // Draw console
    {
        trace!("console::draw");
        debug_systems
            .console
            .lock()
            .unwrap()
            .draw(window, gres, config);
    }
}

macro_rules! add_msg {
    ($game_state: expr, $msg: expr) => {
        $game_state
            .debug_systems
            .debug_ui
            .get_overlay(sid!("msg"))
            .add_line($msg)
    };
}

fn handle_debug_actions(game_state: &mut Game_State, game_res: &mut Game_Resources) {
    use inle_app::debug_systems::Overlay_Shown;

    let actions = &game_state.input.processed.game_actions;
    // @Speed: eventually we want to replace all the *name == sid with a const sid function, to allow doing
    // (sid!("game_speed_up"), Action_Kind::Pressed) => { ... }
    for action in actions {
        match action {
            (name, Action_Kind::Pressed) if *name == sid!("calipers") => {
                let camera_xform = inle_math::transform::Transform2D::default();
                game_state.debug_systems.calipers.start_measuring_dist(
                    &game_state.window,
                    &camera_xform,
                    &game_state.input,
                );
            }
            (name, Action_Kind::Released) if *name == sid!("calipers") => {
                game_state.debug_systems.calipers.end_measuring();
            }
            (name, Action_Kind::Pressed)
                if *name == sid!("game_speed_up") || *name == sid!("game_speed_down") =>
            {
                let mut ts = game_state.time.time_scale;
                if action.0 == sid!("game_speed_up") {
                    ts *= 2.0;
                } else {
                    ts *= 0.5;
                }
                ts = inle_math::math::clamp(ts, 0.001, 32.0);
                if ts > 0.0 {
                    game_state.time.time_scale = ts;
                }
                add_msg!(
                    game_state,
                    &format!("Time scale: {:.3}", game_state.time.time_scale)
                );
            }
            (name, Action_Kind::Pressed) if *name == sid!("pause_toggle") => {
                game_state.time.pause_toggle();
                inle_win::window::set_key_repeat_enabled(
                    &mut game_state.window,
                    game_state.time.paused,
                );
                add_msg!(
                    game_state,
                    if game_state.time.paused {
                        "Paused"
                    } else {
                        "Resumed"
                    }
                );
            }
            (name, Action_Kind::Pressed) if *name == sid!("step_sim") => {}
            (name, Action_Kind::Pressed) if *name == sid!("toggle_trace_overlay") => {
                let show_trace = game_state.debug_systems.show_overlay;
                let show_trace = show_trace != Overlay_Shown::Trace;
                if show_trace {
                    game_state.debug_systems.show_overlay = Overlay_Shown::Trace;
                } else {
                    game_state.debug_systems.show_overlay = Overlay_Shown::None;
                }
                game_state
                    .debug_systems
                    .debug_ui
                    .set_overlay_enabled(sid!("trace"), show_trace);
            }
            (name, Action_Kind::Pressed) if *name == sid!("toggle_threads_overlay") => {
                let show_threads = game_state.debug_systems.show_overlay;
                let show_threads = show_threads != Overlay_Shown::Threads;
                if show_threads {
                    game_state
                        .debug_systems
                        .debug_ui
                        .set_overlay_enabled(sid!("trace"), false);
                    game_state.debug_systems.show_overlay = Overlay_Shown::Threads;
                } else {
                    game_state.debug_systems.show_overlay = Overlay_Shown::None;
                }
            }
            (name, Action_Kind::Pressed) if *name == sid!("move_camera_to_origin") => {
                add_msg!(game_state, "Moved camera to origin");
            }
            (name, Action_Kind::Released) if *name == sid!("toggle_camera_on_player") => {}
            _ => {}
        }
    }
}

pub fn set_traced_fn(debug_systems: &mut Debug_Systems, fn_name: String) {
    debug_systems.traced_fn = fn_name.clone();
    let graph = debug_systems.debug_ui.get_graph(sid!("fn_profile"));
    graph.cfg.title = Some(fn_name);
    graph.data.points.clear();
    graph.selected_point = None;
}