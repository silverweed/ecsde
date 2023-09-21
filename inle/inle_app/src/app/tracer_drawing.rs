use crate::debug_systems::Debug_Systems;
use inle_alloc::temp::Temp_Allocator;
use inle_cfg::{Cfg_Var, Config};
use inle_common::colors;
use inle_common::units::format_bytes_pretty;
use inle_core::time;
use inle_debug::graph;
use inle_debug::overlay::Debug_Overlay;
use inle_diagnostics::tracer::{self, Trace_Tree, Tracer_Node_Final};
use inle_math::transform::Transform2D;
use std::borrow::Cow;
use std::collections::HashMap;
use std::thread::ThreadId;
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

pub fn update_trace_tree_overlay(
    debug_systems: &mut Debug_Systems,
    config: &Config,
    frame_alloc: &mut Temp_Allocator,
) {
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

    let scroller = &debug_systems.debug_ui.frame_scroller;
    let debug_log = &mut debug_systems.log;
    let frame = scroller.get_real_selected_frame();
    let debug_frame = debug_log.get_frame(frame);
    if debug_frame.is_none() {
        // This can happen if the frame scroller is never updated.
        return;
    }
    let traces = &debug_frame.unwrap().traces;
    let overlay = debug_systems.debug_ui.get_overlay(sid!("trace"));

    overlay.clear();

    let total_traced_time = tracer::total_traced_time(traces);
    let mut trace_trees = tracer::build_trace_trees(traces);
    tracer::sort_trace_trees(&mut trace_trees);

    overlay.cfg.font_size = Cfg_Var::new("engine/debug/trace/font_size", config);

    let prune_duration_ms =
        Cfg_Var::<f32>::new("engine/debug/trace/prune_duration_ms", config).read(config);
    let prune_duration = Duration::from_secs_f32(prune_duration_ms * 0.001);

    overlay
        .add_line(&format!(
            "frame {} | debug_log_mem {} | temp_mem_max_usage {} / {}",
            frame,
            format_bytes_pretty(debug_log.mem_used),
            format_bytes_pretty(frame_alloc.high_water_mark),
            format_bytes_pretty(frame_alloc.cap)
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

pub fn update_trace_flat_overlay(
    debug_systems: &mut Debug_Systems,
    config: &Config,
    frame_alloc: &mut Temp_Allocator,
) {
    let scroller = &debug_systems.debug_ui.frame_scroller;
    let debug_log = &mut debug_systems.log;
    let frame = scroller.get_real_selected_frame();
    let overlay = debug_systems.debug_ui.get_overlay(sid!("trace"));

    overlay.clear();

    overlay.cfg.font_size = Cfg_Var::new("engine/debug/trace/font_size", config);

    let prune_duration_ms =
        Cfg_Var::<f32>::new("engine/debug/trace/prune_duration_ms", config).read(config);
    let prune_duration = Duration::from_secs_f32(prune_duration_ms * 0.001);

    overlay
        .add_line(&format!(
            "frame {} | debug_log_mem {} | temp_mem_max_usage {} / {}",
            frame,
            format_bytes_pretty(debug_log.mem_used),
            format_bytes_pretty(frame_alloc.high_water_mark),
            format_bytes_pretty(frame_alloc.cap)
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

// @Incomplete: currently we're passing trace_roots calculated for the latest frame,
// but we should actually be using the frame selected from the scroller and saved in Debug_Log,
// to allow revisiting old frames.
// To do that, we must first save the relevant data in Debug_Log, which currently does not
// contain the start/end times.
pub fn update_thread_overlay(
    debug_systems: &mut Debug_Systems,
    app_config: &crate::app_config::App_Config,
    trace_roots: &[(ThreadId, tracer::Tracer_Node)],
) {
    // @Copypaste from update_trace_tree_overlay
    //let scroller = &engine_state.debug_systems.debug_ui.frame_scroller;
    //let debug_log = &mut engine_state.debug_systems.log;
    //let frame = scroller.get_real_selected_frame();
    //let debug_frame = debug_log.get_frame(frame);
    //if debug_frame.is_none() {
    //// This can happen if the frame scroller is never updated.
    //return;
    //}
    //let traces = &debug_frame.unwrap().traces;

    let painter = &mut debug_systems.global_painter;
    let (win_w, win_h) = (
        app_config.target_win_size.0 as f32,
        app_config.target_win_size.1 as f32,
    );
    let outer_width = win_w * 0.5;
    let outer_height = win_h * 0.5;
    let outer_rect_pos = -v2!(outer_width, outer_height) * 0.5;
    painter.add_rect(
        v2!(outer_width, outer_height),
        &Transform2D::from_pos(outer_rect_pos),
        colors::rgba(30, 30, 60, 220),
    );

    if trace_roots.is_empty() {
        return;
    }

    let (first_t, last_t) =
        min_max_by_key(trace_roots.iter(), |(_, t)| (t.info.start_t, t.info.end_t)).unwrap();
    let tot_duration = last_t.duration_since(first_t);
    let colors = [colors::WHITE, colors::RED, colors::GREEN]; // @Temporary
    let mut col_idx = 0;
    let mut thread_y_offsets: HashMap<ThreadId, f32> = HashMap::default();
    let mut next_y_offset = 0.0;
    for (tid, root) in trace_roots {
        let offset = if first_t == root.info.start_t {
            0.
        } else {
            inle_core::time::duration_ratio(
                &root.info.start_t.duration_since(first_t),
                &tot_duration,
            )
        };
        let length = inle_core::time::duration_ratio(
            &root.info.end_t.duration_since(root.info.start_t),
            &tot_duration,
        );
        debug_assert!((0. ..=1.).contains(&offset));
        debug_assert!((0. ..=1.).contains(&length));

        let abs_width = v2!(length * outer_width, 10.);
        let yoff = *thread_y_offsets.entry(*tid).or_insert_with(|| {
            let off = next_y_offset;
            next_y_offset += 12.;
            off
        });

        let abs_offset = v2!(offset * outer_width, yoff);

        painter.add_rect(
            abs_width,
            &Transform2D::from_pos(outer_rect_pos + abs_offset),
            colors[col_idx],
        );
        col_idx = (col_idx + 1) % colors.len();
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

fn min_max_by_key<T, I, F>(iter: I, mut f: F) -> Option<(T, T)>
where
    T: Ord + Clone,
    I: Iterator,
    F: FnMut(&I::Item) -> (T, T),
{
    let mut min_a = None;
    let mut max_b = None;
    for item in iter {
        let (a, b) = f(&item);
        if let Some(cur_a) = min_a.clone() {
            if a < cur_a {
                min_a = Some(a);
            }
        } else {
            min_a = Some(a);
        }

        if let Some(cur_b) = max_b.clone() {
            if b > cur_b {
                max_b = Some(b);
            }
        } else {
            max_b = Some(b);
        }
    }

    Some((min_a?, max_b?))
}
