use crate::app::Engine_State;
use inle_cfg::Cfg_Var;
use inle_common::colors;
use inle_common::units::format_bytes_pretty;
use inle_core::time;
use inle_debug::graph;
use inle_debug::overlay::Debug_Overlay;
use inle_diagnostics::tracer::{self, Trace_Tree, Tracer_Node_Final};
use std::borrow::Cow;
use std::time::Duration;

fn add_tracer_node_line(
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
    let width = 40 - indent;
    let tag = if node.info.tag.len() <= width {
        Cow::Borrowed(node.info.tag)
    } else {
        Cow::Owned(node.info.tag.chars().take(width - 1).collect::<String>() + "~")
    };
    line.push_str(&format!(
        "{:width$}: {:>6.3}ms ({:3}%): {:>7}: {:6.3}ms",
        tag,
        duration_ms,
        (ratio * 100.0) as u32,
        n_calls,
        duration_ms / n_calls as f32,
        width = width,
    ));

    let bg_col = colors::Color { a: 50, ..color };
    overlay
        .add_line(&line)
        .with_color(color)
        .with_bg_rect_fill(bg_col, ratio)
        .with_metadata(sid!("full_tag"), node.info.tag.to_string());
}

pub fn update_trace_tree_overlay(engine_state: &mut Engine_State) {
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

        add_tracer_node_line(tree.node, total_traced_time, indent, overlay);
        for t in &tree.children {
            add_tree_lines(t, total_traced_time, indent + 1, overlay, prune_duration);
        }
    }

    let scroller = &engine_state.debug_systems.debug_ui.frame_scroller;
    let debug_log = &mut engine_state.debug_systems.log;
    let frame = scroller.get_real_selected_frame();
    let debug_frame = debug_log.get_frame(frame);
    if debug_frame.is_none() {
        // This can happen if the frame scroller is never updated.
        return;
    }
    let traces = &debug_frame.unwrap().traces;
    let overlay = engine_state
        .debug_systems
        .debug_ui
        .get_overlay(sid!("trace"));

    overlay.clear();

    let total_traced_time = tracer::total_traced_time(traces);
    let mut trace_trees = tracer::build_trace_trees(traces);
    tracer::sort_trace_trees(&mut trace_trees);

    overlay.cfg.font_size = Cfg_Var::new("engine/debug/trace/font_size", &engine_state.config);

    let prune_duration_ms =
        Cfg_Var::<f32>::new("engine/debug/trace/prune_duration_ms", &engine_state.config)
            .read(&engine_state.config);
    let prune_duration = Duration::from_secs_f32(prune_duration_ms * 0.001);

    overlay
        .add_line(&format!(
            "frame {} | debug_log_mem {} | temp_mem_max_usage {} / {}",
            frame,
            format_bytes_pretty(debug_log.mem_used),
            format_bytes_pretty(engine_state.frame_alloc.high_water_mark),
            format_bytes_pretty(engine_state.frame_alloc.cap)
        ))
        .with_color(colors::rgb(144, 144, 144));
    overlay
        .add_line(&format!(
            "{:<39}: {:<15}: {:7}: {:>7}",
            "procedure_name", "tot_time", "n_calls", "t/call"
        ))
        .with_color(colors::rgb(204, 0, 102));
    overlay
        .add_line(&format!("{:─^80}", ""))
        .with_color(colors::rgba(60, 60, 60, 180));
    for tree in &trace_trees {
        add_tree_lines(tree, &total_traced_time, 0, overlay, &prune_duration);
    }
}

pub fn update_trace_flat_overlay(engine_state: &mut Engine_State) {
    let scroller = &engine_state.debug_systems.debug_ui.frame_scroller;
    let debug_log = &mut engine_state.debug_systems.log;
    let frame = scroller.get_real_selected_frame();
    let overlay = engine_state
        .debug_systems
        .debug_ui
        .get_overlay(sid!("trace"));

    overlay.clear();

    overlay.cfg.font_size = Cfg_Var::new("engine/debug/trace/font_size", &engine_state.config);

    let prune_duration_ms =
        Cfg_Var::<f32>::new("engine/debug/trace/prune_duration_ms", &engine_state.config)
            .read(&engine_state.config);
    let prune_duration = Duration::from_secs_f32(prune_duration_ms * 0.001);

    overlay
        .add_line(&format!(
            "frame {} | debug_log_mem {} | temp_mem_max_usage {} / {}",
            frame,
            format_bytes_pretty(debug_log.mem_used),
            format_bytes_pretty(engine_state.frame_alloc.high_water_mark),
            format_bytes_pretty(engine_state.frame_alloc.cap)
        ))
        .with_color(colors::rgb(144, 144, 144));
    overlay
        .add_line(&format!(
            "{:<39}: {:<15}: {:7}: {:>7}",
            "procedure_name", "tot_time", "n_calls", "t/call"
        ))
        .with_color(colors::rgb(204, 0, 102));
    overlay
        .add_line(&format!("{:─^80}", ""))
        .with_color(colors::rgba(60, 60, 60, 180));

    let traces = &debug_log.get_frame(frame).unwrap().traces;
    let total_traced_time = tracer::total_traced_time(traces);
    let mut traces = tracer::flatten_traces(traces).collect::<Vec<_>>();
    traces.sort_by(|a, b| b.info.tot_duration().cmp(&a.info.tot_duration()));

    for node in traces
        .iter()
        .filter(|n| n.info.tot_duration() > prune_duration)
    {
        add_tracer_node_line(node, &total_traced_time, 0, overlay);
    }
}

pub fn update_graph_traced_fn(
    traces: impl Iterator<Item = Tracer_Node_Final>, // NOTE: these must be flattened!
    graph: &mut graph::Debug_Graph_View,
    time: Duration,
    traced_fn: &str,
    cur_frame: u64,
) {
    // @Incomplete: make this configurable
    const TIME_LIMIT: f32 = 20.0;

    let fn_tot_time = traces
        .filter_map(|t| {
            if t.info.tag == traced_fn {
                Some(t.info.tot_duration().as_secs_f32() * 1000.)
            } else {
                None
            }
        })
        .sum();

    // @Robustness: we're demoting the u64 to a u32 just for laziness!
    graph::add_point_and_scroll_with_metadata(
        graph,
        time,
        TIME_LIMIT,
        fn_tot_time,
        &[(sid!("real_frame"), (cur_frame as u32).into())],
    );
}
