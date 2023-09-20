use inle_cfg::Cfg_Var;

pub fn update_traces(show_overlay: inle_app::debug_systems::Overlay_Shown, refresh_rate: Cfg_Var<f32>, 
                 debug_systems: &mut inle_app::debug_systems::Debug_Systems, config: &inle_cfg::config::Config,
                 time: &inle_core::time::Time, cur_frame: u64, frame_alloc: &mut inle_alloc::temp::Temp_Allocator,
                 ) 
{
    use inle_app::debug_systems::Overlay_Shown;
    use inle_diagnostics::{prelude, tracer::Tracer_Node};
    use std::thread::ThreadId;
    use inle_app::app::tracer_drawing;

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
    let is_threads_view = matches!(show_overlay, Overlay_Shown::Threads);
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
    /*
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

            let flattened_traces = tracer::flatten_traces(&final_traces);
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
    */
}
