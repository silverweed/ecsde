#![allow(non_camel_case_types)]

#[macro_use]
extern crate inle_diagnostics;

#[macro_use]
extern crate inle_math;

use std::ffi::c_char;

use inle_cfg::Cfg_Var;
use inle_input::core_actions::Core_Action;

pub struct Game_State {
    should_quit: bool,
    env: inle_core::env::Env_Info,
    config: inle_cfg::config::Config,

    time: inle_core::time::Time,
    cur_frame: u64,

    frame_alloc: inle_alloc::temp::Temp_Allocator,

    window: inle_gfx::render_window::Render_Window_Handle,
    input: inle_input::input_state::Input_State,

    default_font: inle_resources::gfx::Font_Handle,

    debug_sys: inle_app::debug_systems::Debug_Systems,
}

pub struct Game_Resources<'r> {
    pub gfx: inle_resources::gfx::Gfx_Resources<'r>,
    pub audio: inle_resources::audio::Audio_Resources<'r>,
    pub shader_cache: inle_resources::gfx::Shader_Cache<'r>,
}

#[repr(C)]
pub struct Game_Bundle<'r> {
    pub game_state: *mut Game_State,
    pub game_resources: *mut Game_Resources<'r>,
}

#[no_mangle]
pub unsafe extern "C" fn game_init<'a>(_args: *const *const c_char, _args_count: usize) -> Game_Bundle<'a> {
    let mut game_state = internal_game_init();
    let mut game_res = create_game_resources();

    game_post_init(&mut *game_state, &mut *game_res);

    Game_Bundle {
        game_state: Box::into_raw(game_state),
        game_resources: Box::into_raw(game_res),
    }
}

#[no_mangle]
pub unsafe extern "C" fn game_update(game_state: *mut Game_State, game_res: *mut Game_Resources<'_>) -> bool {
    let game_state = &mut *game_state;
    let game_res = &mut *game_res;

    game_state.debug_sys.log.start_frame();
    inle_gfx::render_window::start_new_frame(&mut game_state.window);

    //
    // Input
    //
    process_input(game_state);

    //
    // Render
    //
    let win = &mut game_state.window;
    inle_gfx::render_window::clear(win);
    let font = game_res.gfx.get_font(game_state.default_font);
    let txt = inle_gfx::render::create_text(win, "Hello Minigame!", font, 42);
    inle_gfx::render::render_text(win, &txt, inle_common::colors::GREEN, v2!(100., 100.));
    inle_win::window::display(win);

    let refresh_rate = Cfg_Var::new("engine/debug/trace/refresh_rate", &game_state.config);
    update_traces(inle_app::debug_systems::Overlay_Shown::Trace, refresh_rate, 
                 &mut game_state.debug_sys, &game_state.config,
                 &game_state.time, game_state.cur_frame, &mut game_state.frame_alloc);


    game_state.frame_alloc.dealloc_all();

    !game_state.should_quit
}

#[no_mangle]
pub unsafe extern "C" fn game_shutdown(_game_state: *mut Game_State, _game_res: *mut Game_Resources) {}

/*
#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn game_unload(_game_state: *mut Game_State, _game_res: *mut Game_Resources) {}

#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn game_reload(_game_state: *mut Game_State, _game_res: *mut Game_Resources) {}
*/

//
// Init
//
fn internal_game_init() -> Box<Game_State> {
    use inle_core::env::Env_Info;

    let mut loggers = unsafe { inle_diagnostics::log::create_loggers() };
    inle_diagnostics::log::add_default_logger(&mut loggers);

    linfo!("Hello logs!");

    let env = Env_Info::gather().unwrap();
    let config = inle_cfg::Config::new_from_dir(&env.cfg_root);

    let window = {
        let win_width: Cfg_Var<i32> = Cfg_Var::new("engine/window/width", &config);
        let win_height: Cfg_Var<i32> = Cfg_Var::new("engine/window/height", &config);
        let win_title: Cfg_Var<String> = Cfg_Var::new("engine/window/title", &config);
        let target_win_size = (win_width.read(&config) as u32, win_height.read(&config) as u32);
        let window_create_args = inle_win::window::Create_Window_Args { vsync: true };
        let window = inle_win::window::create_window(&window_create_args, target_win_size, win_title.read(&config));
        inle_gfx::render_window::create_render_window(window)
    };

    let input = inle_input::input_state::create_input_state(&env);

    let seed = inle_core::rand::new_random_seed().unwrap();
    let debug_sys = inle_app::debug_systems::Debug_Systems::new(&config, seed);
    let time = inle_core::time::Time::default();
    let frame_alloc = inle_alloc::temp::Temp_Allocator::with_capacity(inle_common::units::gigabytes(1));

    Box::new(Game_State { 
        env,
        config,
        window, 
        time,
        input,
        cur_frame: 0,
        frame_alloc,
        should_quit: false,
        default_font: None,
        debug_sys,
    })
}

fn create_game_resources<'a>() -> Box<Game_Resources<'a>> {
    let gfx = inle_resources::gfx::Gfx_Resources::new();
    let audio = inle_resources::audio::Audio_Resources::new();
    let shader_cache = inle_resources::gfx::Shader_Cache::new();
    Box::new(Game_Resources {
        gfx,
        audio,
        shader_cache,
    })
}

// Used to initialize stuff that needs resources
fn game_post_init(game_state: &mut Game_State, game_res: &mut Game_Resources<'_>) {
    let font_name = inle_cfg::Cfg_Var::<String>::new("engine/debug/ui/font", &game_state.config);
    game_state.default_font = game_res.gfx
        .load_font(&inle_resources::gfx::font_path(
            &game_state.env,
            font_name.read(&game_state.config),
        ));
}

//
// Input
//
fn process_input(game_state: &mut Game_State) {
    inle_input::input_state::update_raw_input(
        &mut game_state.window,
        &mut game_state.input.raw,
    );
    let process_game_actions = true;
    inle_input::input_state::process_raw_input(
        &game_state.input.raw,
        &game_state.input.bindings,
        &mut game_state.input.processed,
        process_game_actions,
    );
    if handle_core_actions(&mut game_state.window, &mut game_state.input) {
        game_state.should_quit = true;
    }
}


fn handle_core_actions(window: &mut inle_gfx::render_window::Render_Window_Handle, input: &mut inle_input::input_state::Input_State) -> bool {
    for action in &input.processed.core_actions {
        match action {
            Core_Action::Quit => return true,
            Core_Action::Resize(new_width, new_height) => {
                inle_gfx::render_window::resize_keep_ratio(window, *new_width, *new_height);
            }
            Core_Action::Focus_Lost => {
                input.raw.kb_state.modifiers_pressed = 0;
                inle_input::mouse::reset_mouse_state(&mut input.raw.mouse_state);
            }
            _ => unimplemented!(),
        }
    }
    false
}


fn update_traces(show_overlay: inle_app::debug_systems::Overlay_Shown, refresh_rate: Cfg_Var<f32>, 
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
