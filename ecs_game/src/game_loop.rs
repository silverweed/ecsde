use super::{Game_Resources, Game_State};
use crate::states::state::Game_State_Args;
use inle_alloc::temp::*;
use inle_app::{app, render_system};
use inle_common::colors;
use inle_core::time;
use inle_gfx::render_window::Render_Window_Handle;
use inle_math::transform::Transform2D;
use inle_physics::physics;
use inle_resources::gfx::Gfx_Resources;
use std::time::Duration;

#[cfg(debug_assertions)]
use {
    inle_common::paint_props::Paint_Properties,
    inle_common::stringid::String_Id,
    inle_debug::painter::Debug_Painter,
    inle_ecs::components::base::C_Spatial2D,
    inle_ecs::ecs_world::Ecs_World,
    inle_gfx::render_window,
    inle_input::input_state::Input_State,
    inle_input::mouse,
    inle_math::angle::rad,
    inle_math::shapes::{Arrow, Circle, Line},
    inle_math::vector::Vec2f,
    inle_physics::phys_world::Physics_World,
    inle_win::window,
    std::collections::HashMap,
    std::convert::TryInto,
};

pub fn tick_game<'a, 's, 'r>(
    game_state: &'a mut Game_State<'s>,
    game_resources: &'a mut Game_Resources<'r>,
) -> Result<(), Box<dyn std::error::Error>>
where
    'r: 's,
    's: 'a,
{
    trace!("tick_game");

    game_state.engine_state.cur_frame += 1;
    let time = &mut game_state.engine_state.time;
    time.update();
    let dt = time.dt();
    let real_dt = time.real_dt();

    let target_time_per_frame = Duration::from_micros(
        (game_state
            .cvars
            .gameplay_update_tick_ms
            .read(&game_state.engine_state.config)
            * 1000.0) as u64,
    );

    let update_dt = time::mul_duration(
        &target_time_per_frame,
        time.time_scale * (!time.paused as u32 as f32),
    );

    #[cfg(debug_assertions)]
    let debug_systems = &mut game_state.engine_state.debug_systems;

    let mut_in_debug!(replaying) = false;

    // Update input
    {
        trace!("input_state::update");

        let process_game_actions;

        #[cfg(debug_assertions)]
        {
            process_game_actions = debug_systems.console.lock().unwrap().status
                != inle_debug::console::Console_Status::Open;
        }
        #[cfg(not(debug_assertions))]
        {
            process_game_actions = true;
        }

        // Update the raw input. This may be overwritten later by replay data, but its core_events won't.
        inle_input::input_state::update_raw_input(
            &mut game_state.window,
            &mut game_state.engine_state.input_state.raw,
        );

        #[cfg(debug_assertions)]
        if let Some(rip) = game_state.engine_state.replay_input_provider.as_mut() {
            if rip.has_more_input() {
                replaying = true;

                if let Some(mut raw_input) =
                    rip.get_replayed_input_for_frame(game_state.engine_state.cur_frame)
                {
                    std::mem::swap(&mut game_state.engine_state.input_state.raw, &mut raw_input);
                    // Preserve the realtime core events.
                    // Note: now raw_input contains the realtime events
                    game_state
                        .engine_state
                        .input_state
                        .raw
                        .events
                        .extend(raw_input.core_events.iter().cloned());
                    game_state.engine_state.input_state.raw.core_events = raw_input.core_events;
                } else {
                    game_state.engine_state.input_state.raw.events =
                        game_state.engine_state.input_state.raw.core_events.clone();
                }
            } else {
                game_state.engine_state.replay_input_provider = None;
            }
        }

        inle_input::input_state::process_raw_input(
            &game_state.engine_state.input_state.raw,
            &game_state.engine_state.input_state.bindings,
            &mut game_state.engine_state.input_state.processed,
            process_game_actions,
        );
    }

    #[cfg(debug_assertions)]
    {
        use crate::debug::console_executor;

        trace!("console::update");
        let mut console = game_state
            .engine_state
            .debug_systems
            .console
            .lock()
            .unwrap();
        if console.status == inle_debug::console::Console_Status::Open {
            console.update(&game_state.engine_state.input_state);

            let mut output = vec![];
            let mut commands = vec![];
            while let Some(cmd) = console.pop_enqueued_cmd() {
                if !cmd.is_empty() {
                    commands.push(cmd);
                }
            }

            drop(console);

            for cmd in commands {
                let maybe_output = console_executor::execute(
                    &cmd,
                    &mut game_state.engine_state,
                    &mut game_state.gameplay_system,
                );
                if let Some(out) = maybe_output {
                    output.push(out);
                }
            }

            for (out, color) in output {
                game_state
                    .engine_state
                    .debug_systems
                    .console
                    .lock()
                    .unwrap()
                    .output_line(format!(">> {}", out), color);
            }
        }
    }

    #[cfg(debug_assertions)]
    let debug_systems = &mut game_state.engine_state.debug_systems;

    // Frame scroller events
    #[cfg(debug_assertions)]
    {
        let scroller = &mut debug_systems.debug_ui.frame_scroller;
        let prev_selected_frame = scroller.cur_frame;
        let prev_selected_second = scroller.cur_second;
        let was_manually_selected = scroller.manually_selected;

        scroller.handle_events(&game_state.engine_state.input_state.raw.events);

        if scroller.cur_frame != prev_selected_frame
            || scroller.cur_second != prev_selected_second
            || was_manually_selected != scroller.manually_selected
        {
            game_state.engine_state.time.paused = scroller.manually_selected;
            debug_systems.trace_overlay_update_t = 0.;
        }
    }

    // Handle core actions (resize, quit, ..)
    {
        trace!("app::handle_core_actions");
        if app::handle_core_actions(
            &game_state
                .engine_state
                .input_state
                .processed
                .core_actions
                .split_off(0),
            &mut game_state.window,
            &mut game_state.engine_state,
        ) {
            game_state.engine_state.should_close = true;
            return Ok(());
        }
    }

    #[cfg(debug_assertions)]
    let debug_systems = &mut game_state.engine_state.debug_systems;

    #[cfg(debug_assertions)]
    let mut collision_debug_data = HashMap::new();

    {
        #[cfg(debug_assertions)]
        {
            let input_state = &game_state.engine_state.input_state;

            update_joystick_debug_overlay(
                debug_systems.debug_ui.get_overlay(sid!("joysticks")),
                &input_state.raw.joy_state,
                game_state.gameplay_system.input_cfg,
                &game_state.engine_state.config,
            );

            // Record replay data (only if we're not already playing back a replay).
            let recording = !replaying && debug_systems.replay_recording_system.is_recording();
            if recording {
                debug_systems
                    .replay_recording_system
                    .update(&input_state.raw, game_state.engine_state.cur_frame);
            }

            update_record_debug_overlay(
                debug_systems.debug_ui.get_overlay(sid!("record")),
                recording,
                replaying,
            );
        }

        // Update states
        {
            trace!("state_mgr::handle_actions");

            // Note: we clone the actions here to ease the implementation of handle_actions and avoid
            // ownership errors due to mutably borrowing engine_state.
            let actions = game_state
                .engine_state
                .input_state
                .processed
                .game_actions
                .clone();
            let mut args = Game_State_Args {
                engine_state: &mut game_state.engine_state,
                gameplay_system: &mut game_state.gameplay_system,
                window: &mut game_state.window,
                game_resources,
                level_batches: &mut game_state.level_batches,
                cvars: &game_state.cvars,
            };
            game_state.state_mgr.handle_actions(&actions, &mut args);
            if game_state.engine_state.should_close {
                return Ok(());
            }
        }

        #[cfg(debug_assertions)]
        {
            use inle_debug::console::Console_Status;
            use inle_input::input_state::Action_Kind;

            let actions = &game_state.engine_state.input_state.processed.game_actions;

            if actions.contains(&(sid!("toggle_console"), Action_Kind::Pressed)) {
                game_state
                    .engine_state
                    .debug_systems
                    .console
                    .lock()
                    .unwrap()
                    .toggle();
            }

            if actions.contains(&(sid!("calipers"), Action_Kind::Pressed)) {
                if let Some(level) = game_state.gameplay_system.levels.first_active_level() {
                    game_state
                        .engine_state
                        .debug_systems
                        .calipers
                        .start_measuring_dist(
                            &game_state.window,
                            &level.get_camera_transform(),
                            &game_state.engine_state.input_state,
                        );
                }
            } else if actions.contains(&(sid!("calipers"), Action_Kind::Released)) {
                game_state
                    .engine_state
                    .debug_systems
                    .calipers
                    .end_measuring();
            }

            window::set_key_repeat_enabled(
                &mut game_state.window,
                game_state
                    .engine_state
                    .debug_systems
                    .console
                    .lock()
                    .unwrap()
                    .status
                    == Console_Status::Open,
            );
        }

        // Update game systems
        {
            trace!("game_update");

            game_state
                .gameplay_system
                .realtime_update(&real_dt, &game_state.engine_state);

            // @Cleanup: where do we put this? Do we want this inside gameplay_system?
            {
                trace!("state_mgr::update");

                let mut args = Game_State_Args {
                    engine_state: &mut game_state.engine_state,
                    gameplay_system: &mut game_state.gameplay_system,
                    window: &mut game_state.window,
                    game_resources,
                    level_batches: &mut game_state.level_batches,
                    cvars: &game_state.cvars,
                };
                game_state.state_mgr.update(&mut args, &dt, &real_dt);
            }

            if !game_state.engine_state.time.paused {
                game_state.accumulated_update_time += dt;

                let max_time_budget = Duration::from_micros(
                    (game_state
                        .cvars
                        .gameplay_max_time_budget_ms
                        .read(&game_state.engine_state.config)
                        * 1000.0) as u64,
                );
                let mut n_updates = 0;
                let mut time_spent_in_update = Duration::default();

                // We must save the input state for later since it gets cleared in the simulation update loop
                // @Cleanup: this kinda sucks, try to find a better solution
                let mut orig_input_state =
                    inle_input::input_state::Input_State_Restore_Point::default();

                let do_update_physics;
                #[cfg(debug_assertions)]
                {
                    do_update_physics = game_state
                        .debug_cvars
                        .update_physics
                        .read(&game_state.engine_state.config);
                }
                #[cfg(not(debug_assertions))]
                {
                    do_update_physics = true;
                }

                while game_state.accumulated_update_time >= update_dt {
                    let update_start = std::time::Instant::now();

                    game_state.gameplay_system.update(
                        &update_dt,
                        &mut game_state.engine_state,
                        game_resources,
                        &game_state.window,
                    );

                    if do_update_physics {
                        update_physics(
                            game_state,
                            update_dt,
                            #[cfg(debug_assertions)]
                            &mut collision_debug_data,
                        );
                    }

                    game_state
                        .gameplay_system
                        .late_update(&mut game_state.engine_state.systems.evt_register);

                    game_state.accumulated_update_time -= update_dt;
                    n_updates += 1;

                    time_spent_in_update += update_start.elapsed();
                    if time_spent_in_update > max_time_budget {
                        lerr!(
                            "Time budget for simulation exhausted ({} ms): losing time!",
                            1000.0 * max_time_budget.as_secs_f32()
                        );
                        break;
                    }

                    if n_updates == 1 {
                        orig_input_state = game_state.engine_state.input_state.clear();
                    }
                }

                game_state.n_updates_last_frame = n_updates;
                game_state
                    .engine_state
                    .input_state
                    .restore(orig_input_state);

                let cfg = &game_state.engine_state.config;
                if game_state.cvars.enable_particles.read(cfg) {
                    for _ in 0..n_updates {
                        game_state
                            .engine_state
                            .systems
                            .particle_mgrs
                            .iter_mut()
                            .for_each(|(_, particle_mgr)| {
                                particle_mgr.update(&update_dt, cfg);
                            });
                    }
                }
            }
        }
    }

    // Update audio
    {
        trace!("audio_system_update");
        game_state.engine_state.systems.audio_system.update();
    }

    // We clear batches before update_debug, so debug can draw textures
    {
        trace!("clear_batches");
        inle_gfx::render::batcher::clear_batches(&mut game_state.engine_state.global_batches);
        for batches in game_state.level_batches.values_mut() {
            inle_gfx::render::batcher::clear_batches(batches);
        }
    }

    #[cfg(debug_assertions)]
    update_debug(game_state, game_resources, collision_debug_data, dt);

    update_graphics(game_state, game_resources);
    update_ui(game_state, &game_resources.gfx);

    #[cfg(debug_assertions)]
    update_debug_graphics(game_state, &mut game_resources.gfx, real_dt);

    #[cfg(debug_assertions)]
    {
        game_state.engine_state.config.update();
        game_state.fps_debug.tick(&real_dt);
    }

    {
        trace!("display");
        inle_win::window::display(&mut game_state.window);
    }

    Ok(())
}

fn update_ui(game_state: &mut Game_State, gres: &Gfx_Resources) {
    trace!("update_ui");

    let window = &mut game_state.window;
    let ui_ctx = &mut game_state.engine_state.systems.ui;

    inle_ui::draw_all_ui(window, gres, ui_ctx);
}

fn update_graphics<'a, 's, 'r>(
    game_state: &'a mut Game_State<'s>,
    game_resources: &mut Game_Resources<'r>,
) where
    'r: 's,
    's: 'a,
{
    trace!("update_graphics");

    let window = &mut game_state.window;
    let gres = &mut game_resources.gfx;
    let shader_cache = &mut game_resources.shader_cache;

    #[cfg(debug_assertions)]
    {
        let cur_vsync = inle_win::window::has_vsync(window);
        let desired_vsync = game_state.cvars.vsync.read(&game_state.engine_state.config);
        if desired_vsync != cur_vsync {
            inle_win::window::set_vsync(window, desired_vsync);
        }
    }

    {
        trace!("clear_window");

        let clear_color = colors::color_from_hex(
            game_state
                .cvars
                .clear_color
                .read(&game_state.engine_state.config),
        );
        inle_gfx::render_window::set_clear_color(window, clear_color);
        inle_gfx::render_window::clear(window);
    }

    let cfg = &game_state.engine_state.config;

    #[cfg(debug_assertions)]
    {
        use inle_gfx::light::*;

        if game_state.cvars.ambient_intensity.has_changed(cfg)
            || game_state.cvars.ambient_color.has_changed(cfg)
        {
            let new_amb_intensity = game_state.cvars.ambient_intensity.read(cfg);
            let new_amb_color =
                inle_common::colors::color_from_hex(game_state.cvars.ambient_color.read(cfg));
            game_state
                .gameplay_system
                .levels
                .foreach_active_level(|level| {
                    level
                        .lights
                        .queue_command(Light_Command::Change_Ambient_Light(Ambient_Light {
                            intensity: new_amb_intensity,
                            color: new_amb_color,
                        }));
                });
        }
    }

    let render_cfg = render_system::Render_System_Config {
        clear_color: colors::color_from_hex(game_state.cvars.clear_color.read(cfg) as u32),
        #[cfg(debug_assertions)]
        debug_visualization: get_render_system_debug_visualization(&game_state.debug_cvars, cfg),
    };

    {
        let gameplay_system = &mut game_state.gameplay_system;
        let batches = &mut game_state.level_batches;
        let frame_alloc = &mut game_state.engine_state.frame_alloc;
        gameplay_system.levels.foreach_active_level(|level| {
            let render_args = render_system::Render_System_Update_Args {
                batches: batches.get_mut(&level.id).unwrap(),
                ecs_world: &level.world,
                frame_alloc,
                cfg: render_cfg,
                window,
                camera: &level.get_camera_transform(),
                gres,
                shader_cache,
            };

            render_system::update(render_args);
        });
    }

    // Draw texture batches and particle systems
    {
        use inle_gfx::render::batcher;

        let frame_alloc = &mut game_state.engine_state.frame_alloc;
        let lv_batches = &mut game_state.level_batches;
        let window = &mut game_state.window;
        let enable_shaders = game_state.cvars.enable_shaders.read(cfg);
        let enable_shadows = game_state.cvars.enable_shadows.read(cfg);
        let enable_particles = game_state.cvars.enable_particles.read(cfg);
        let particle_mgrs = &mut game_state.engine_state.systems.particle_mgrs;

        game_state
            .gameplay_system
            .levels
            .foreach_active_level(|level| {
                batcher::draw_batches(
                    window,
                    gres,
                    lv_batches.get_mut(&level.id).unwrap(),
                    shader_cache,
                    &level.get_camera_transform(),
                    &mut level.lights,
                    batcher::Batcher_Draw_Params {
                        enable_shaders,
                        enable_shadows,
                    },
                    frame_alloc,
                );

                if enable_particles {
                    particle_mgrs.get_mut(&level.id).unwrap().render(
                        window,
                        gres,
                        shader_cache,
                        &level.get_camera_transform(),
                        frame_alloc,
                    );
                }
            });

        batcher::draw_batches(
            window,
            gres,
            &mut game_state.engine_state.global_batches,
            shader_cache,
            &Transform2D::default(),
            &mut inle_gfx::light::Lights::default(),
            batcher::Batcher_Draw_Params {
                enable_shaders,
                enable_shadows,
            },
            frame_alloc,
        );
    }
}

#[cfg(debug_assertions)]
fn update_debug_graphics<'a, 's, 'r>(
    game_state: &'a mut Game_State<'s>,
    gres: &'a mut Gfx_Resources<'r>,
    real_dt: Duration,
) {
    // Draw debug calipers
    {
        // @Incomplete @Robustness: first_active_level()?
        let calipers = &game_state.engine_state.debug_systems.calipers;
        let painters = &mut game_state.engine_state.debug_systems.painters;
        if let Some(level) = game_state.gameplay_system.levels.first_active_level() {
            calipers.draw(
                &game_state.window,
                painters.get_mut(&level.id).unwrap(),
                &level.get_camera_transform(),
                &game_state.engine_state.input_state,
            );
        }
    }

    // Draw debug painter (one per active level)
    let painters = &mut game_state.engine_state.debug_systems.painters;
    let window = &mut game_state.window;
    game_state
        .gameplay_system
        .levels
        .foreach_active_level(|level| {
            let painter = painters
                .get_mut(&level.id)
                .unwrap_or_else(|| panic!("Debug painter not found for level {:?}", level.id));
            painter.draw(window, gres, &level.get_camera_transform());
            painter.clear();
        });

    // Global painter
    let painter = &mut game_state.engine_state.debug_systems.global_painter;
    painter.draw(&mut game_state.window, gres, &Transform2D::default());
    painter.clear();

    // Draw debug UI
    {
        let debug_ui = &mut game_state.engine_state.debug_systems.debug_ui;
        let prev_selected = debug_ui.get_graph(sid!("fn_profile")).get_selected_point();

        debug_ui.update_and_draw(
            &real_dt,
            &mut game_state.window,
            gres,
            &game_state.engine_state.input_state,
            &game_state.engine_state.debug_systems.log,
            &game_state.engine_state.config,
            &mut game_state.engine_state.frame_alloc,
        );

        let profile_graph = debug_ui.get_graph(sid!("fn_profile"));
        let cur_selected = profile_graph.get_selected_point();
        if cur_selected != prev_selected {
            game_state.engine_state.time.paused = cur_selected.is_some();
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
        game_state
            .engine_state
            .debug_systems
            .console
            .lock()
            .unwrap()
            .draw(
                &mut game_state.window,
                gres,
                &game_state.engine_state.config,
            );
    }
}

fn update_physics(
    game_state: &mut Game_State,
    update_dt: Duration,
    #[cfg(debug_assertions)] collision_debug_data: &mut HashMap<
        String_Id,
        physics::Collision_System_Debug_Data,
    >,
) {
    trace!("update_physics");

    let gameplay_system = &mut game_state.gameplay_system;
    let levels = &gameplay_system.levels;
    let frame_alloc = &mut game_state.engine_state.frame_alloc;
    let phys_settings = &game_state.engine_state.systems.physics_settings;
    let evt_register = &mut game_state.engine_state.systems.evt_register;

    levels.foreach_active_level(|level| {
        #[cfg(debug_assertions)]
        let coll_debug = collision_debug_data
            .entry(level.id)
            .or_insert_with(physics::Collision_System_Debug_Data::default);

        physics::update_collisions(
            &mut level.world,
            &level.chunks,
            &mut level.phys_world,
            phys_settings,
            evt_register,
            frame_alloc,
            #[cfg(debug_assertions)]
            coll_debug,
        );

        {
            let mut moved = excl_temp_array(frame_alloc);
            crate::movement_system::update(
                &update_dt,
                &mut level.world,
                &level.phys_world,
                &mut moved,
            );

            let moved = unsafe { moved.into_read_only() };
            for mov in &moved {
                level.chunks.update_collider(
                    mov.handle,
                    mov.prev_pos,
                    mov.new_pos,
                    mov.extent,
                    frame_alloc,
                );
            }
        }
    });
}

#[cfg(debug_assertions)]
fn update_debug(
    game_state: &mut Game_State,
    game_resources: &mut Game_Resources,
    collision_debug_data: HashMap<String_Id, physics::Collision_System_Debug_Data>,
    dt: Duration,
) {
    let engine_state = &mut game_state.engine_state;
    let debug_systems = &mut engine_state.debug_systems;

    // Overlays
    let display_overlays = game_state
        .debug_cvars
        .display_overlays
        .read(&engine_state.config);
    let overlays_were_visible = debug_systems.debug_ui.is_overlay_enabled(sid!("time"));
    if display_overlays {
        if !overlays_were_visible {
            set_debug_hud_enabled(&mut debug_systems.debug_ui, true);
        }

        update_time_debug_overlay(
            debug_systems.debug_ui.get_overlay(sid!("time")),
            &engine_state.time,
            game_state.n_updates_last_frame,
        );

        update_fps_debug_overlay(
            debug_systems.debug_ui.get_overlay(sid!("fps")),
            &game_state.fps_debug,
            (1000.
                / game_state
                    .cvars
                    .gameplay_update_tick_ms
                    .read(&engine_state.config)) as u64,
            game_state.cvars.vsync.read(&engine_state.config),
        );

        update_win_debug_overlay(
            debug_systems.debug_ui.get_overlay(sid!("window")),
            &game_state.window,
        );
    } else if overlays_were_visible {
        set_debug_hud_enabled(&mut debug_systems.debug_ui, false);
    }

    let input_state = &engine_state.input_state;
    let draw_mouse_rulers = game_state
        .debug_cvars
        .draw_mouse_rulers
        .read(&engine_state.config);
    // NOTE: this must be always cleared or the mouse position will remain after enabling and disabling the cfg var
    debug_systems.debug_ui.get_overlay(sid!("mouse")).clear();
    if draw_mouse_rulers {
        let painter = &mut debug_systems.global_painter;
        update_mouse_debug_overlay(
            debug_systems.debug_ui.get_overlay(sid!("mouse")),
            painter,
            &game_state.window,
            game_state
                .gameplay_system
                .levels
                .first_active_level()
                .map(|level| level.get_camera_transform()),
            input_state,
        );
    }

    let display_log_window = game_state
        .debug_cvars
        .display_log_window
        .read(&engine_state.config);
    let log_window_enabled = debug_systems
        .debug_ui
        .is_log_window_enabled(sid!("log_window"));
    if display_log_window != log_window_enabled {
        debug_systems
            .debug_ui
            .set_log_window_enabled(sid!("log_window"), display_log_window);
    }

    let draw_fps_graph = game_state
        .debug_cvars
        .draw_fps_graph
        .read(&engine_state.config);
    debug_systems
        .debug_ui
        .set_graph_enabled(sid!("fps"), draw_fps_graph);
    if draw_fps_graph {
        update_graph_fps(
            debug_systems.debug_ui.get_graph(sid!("fps")),
            &engine_state.time,
            &game_state.fps_debug,
        );
    }

    let draw_prev_frame_t_graph = game_state
        .debug_cvars
        .draw_prev_frame_t_graph
        .read(&engine_state.config);
    debug_systems
        .debug_ui
        .set_graph_enabled(sid!("prev_frame_time"), draw_prev_frame_t_graph);
    if draw_prev_frame_t_graph {
        update_graph_prev_frame_t(
            debug_systems.debug_ui.get_graph(sid!("prev_frame_time")),
            &engine_state.time,
            &engine_state.prev_frame_time,
        );
    }

    ////// Per-Level debugs //////
    let painters = &mut debug_systems.painters;
    let debug_ui = &mut debug_systems.debug_ui;
    let target_win_size = engine_state.app_config.target_win_size;

    let is_paused = engine_state.time.paused && !engine_state.time.is_stepping();
    let cvars = &game_state.debug_cvars;
    let draw_entities = cvars.draw_entities.read(&engine_state.config);
    let draw_component_lists = cvars.draw_component_lists.read(&engine_state.config);
    let draw_velocities = cvars.draw_velocities.read(&engine_state.config);
    let draw_entity_prev_frame_ghost = cvars
        .draw_entity_prev_frame_ghost
        .read(&engine_state.config);
    let draw_entity_pos_history = cvars.draw_entity_pos_history.read(&engine_state.config);
    let draw_colliders = cvars.draw_colliders.read(&engine_state.config);
    let draw_debug_grid = cvars.draw_debug_grid.read(&engine_state.config);
    let grid_square_size = cvars.debug_grid_square_size.read(&engine_state.config);
    let grid_font_size = cvars.debug_grid_font_size.read(&engine_state.config);
    let grid_opacity = cvars.debug_grid_opacity.read(&engine_state.config) as u8;
    let draw_world_chunks = cvars.draw_world_chunks.read(&engine_state.config);
    let draw_lights = cvars.draw_lights.read(&engine_state.config);
    let draw_particle_emitters = cvars.draw_particle_emitters.read(&engine_state.config);
    let lv_batches = &mut game_state.level_batches;
    let global_painter = &mut debug_systems.global_painter;
    let window = &mut game_state.window;
    let shader_cache = &mut game_resources.shader_cache;
    let env = &engine_state.env;
    let pos_hist_system = &mut game_state.game_debug_systems.position_history_system;
    let cfg = &engine_state.config;
    let particle_mgrs = &engine_state.systems.particle_mgrs;

    game_state
        .gameplay_system
        .levels
        .foreach_active_level(|level| {
            let debug_painter = painters
                .get_mut(&level.id)
                .unwrap_or_else(|| fatal!("Debug painter not found for level {:?}", level.id));

            if display_overlays {
                update_entities_debug_overlay(debug_ui.get_overlay(sid!("entities")), &level.world);
                update_camera_debug_overlay(
                    debug_ui.get_overlay(sid!("camera")),
                    &level.get_camera_transform(),
                );
                if let Some(cls_debug_data) = collision_debug_data.get(&level.id) {
                    update_physics_debug_overlay(
                        debug_ui.get_overlay(sid!("physics")),
                        cls_debug_data,
                        &level.chunks,
                    );
                }
            }

            if draw_entities {
                debug_draw_transforms(
                    debug_painter,
                    &level.world,
                    window,
                    input_state,
                    &level.get_camera_transform(),
                );
            }

            if draw_velocities {
                debug_draw_velocities(debug_painter, &level.world);
            }

            if draw_entity_prev_frame_ghost {
                let batches = lv_batches.get_mut(&level.id).unwrap();
                debug_draw_entities_prev_frame_ghost(
                    window,
                    batches,
                    shader_cache,
                    env,
                    &mut level.world,
                    is_paused,
                );
            }

            if draw_entity_pos_history {
                pos_hist_system.update(&mut level.world, dt, cfg);
                debug_draw_entities_pos_history(debug_painter, &level.world);
            }

            if draw_component_lists {
                debug_draw_component_lists(debug_painter, &level.world);
            }

            if draw_colliders {
                debug_draw_colliders(debug_painter, &level.world, &level.phys_world);
            }

            if draw_lights {
                debug_draw_lights(global_painter, debug_painter, &level.lights);
            }

            if draw_particle_emitters {
                debug_draw_particle_emitters(debug_painter, particle_mgrs.get(&level.id).unwrap());
            }

            // Debug grid
            if draw_debug_grid {
                debug_draw_grid(
                    debug_painter,
                    &level.get_camera_transform(),
                    target_win_size,
                    grid_square_size,
                    grid_opacity,
                    grid_font_size as _,
                );
            }

            if draw_world_chunks {
                level.chunks.debug_draw(debug_painter);
            }
        });

    // @Cleanup
    if cvars.draw_buf_alloc.read(&engine_state.config) {
        inle_debug::backend_specific_debugs::draw_backend_specific_debug(window, global_painter);
    }
}

#[cfg(debug_assertions)]
fn update_joystick_debug_overlay(
    debug_overlay: &mut inle_debug::overlay::Debug_Overlay,
    joy_state: &inle_input::joystick::Joystick_State,
    input_cfg: crate::input_utils::Input_Config,
    cfg: &inle_cfg::Config,
) {
    use inle_input::joystick;

    debug_overlay.clear();

    let deadzone = input_cfg.joy_deadzone.read(cfg);

    let (real_axes, joy_mask) = inle_input::joystick::get_all_joysticks_axes_values(joy_state);

    for (joy_id, axes) in real_axes.iter().enumerate() {
        if (joy_mask & (1 << joy_id)) != 0 {
            debug_overlay
                .add_line(&format!("> Joy {} <", joy_id))
                .with_color(colors::rgb(235, 52, 216));

            for i in 0u8..joystick::Joystick_Axis::_Count as u8 {
                let axis: joystick::Joystick_Axis = i.try_into().unwrap_or_else(|err| {
                    fatal!("Failed to convert {} to a valid Joystick_Axis: {}", i, err)
                });
                debug_overlay
                    .add_line(&format!("{:?}: {:5.2}", axis, axes[i as usize]))
                    .with_color(if axes[i as usize].abs() > deadzone {
                        colors::GREEN
                    } else {
                        colors::YELLOW
                    });
            }
        }
    }
}

#[cfg(debug_assertions)]
fn update_time_debug_overlay(
    debug_overlay: &mut inle_debug::overlay::Debug_Overlay,
    time: &time::Time,
    n_updates: u32,
) {
    debug_overlay.clear();

    debug_overlay
        .add_line(&format!(
            "[time] game: {:.2}, real: {:.2}, scale: {:.2}, paused: {}, n.upd: {}",
            time.game_time().as_secs_f32(),
            time.real_time().as_secs_f32(),
            time.time_scale,
            if time.paused { "yes" } else { "no" },
            n_updates,
        ))
        .with_color(colors::rgb(100, 200, 200));
}

#[cfg(debug_assertions)]
fn update_fps_debug_overlay(
    debug_overlay: &mut inle_debug::overlay::Debug_Overlay,
    fps: &inle_debug::fps::Fps_Counter,
    target_fps: u64,
    vsync: bool,
) {
    debug_overlay.clear();
    debug_overlay
        .add_line(&format!(
            "FPS: {} (target ~{}, vsync {})",
            fps.get_fps() as u32,
            target_fps,
            if vsync { "on" } else { "off" },
        ))
        .with_color(colors::rgba(180, 180, 180, 200));
}

#[cfg(debug_assertions)]
fn update_mouse_debug_overlay(
    debug_overlay: &mut inle_debug::overlay::Debug_Overlay,
    painter: &mut Debug_Painter,
    window: &Render_Window_Handle,
    camera: Option<Transform2D>,
    input_state: &Input_State,
) {
    let (win_w, win_h) = window::get_window_target_size(window);
    let (win_w, win_h) = (win_w as i32, win_h as i32);
    let pos = mouse::mouse_pos_in_window(window, &input_state.raw.mouse_state);
    debug_overlay.position = Vec2f::from(pos) + v2!(5., -15.);
    let overlay_size = debug_overlay.bounds().size();
    debug_overlay.position.x =
        inle_math::math::clamp(debug_overlay.position.x, 0., win_w as f32 - overlay_size.x);
    debug_overlay.position.y =
        inle_math::math::clamp(debug_overlay.position.y, overlay_size.y, win_h as f32);
    debug_overlay
        .add_line(&format!("s {},{}", pos.x, pos.y))
        .with_color(colors::rgba(220, 220, 220, 220));
    if let Some(camera) = camera {
        let wpos = render_window::mouse_pos_in_world(window, &input_state.raw.mouse_state, &camera);
        debug_overlay
            .add_line(&format!("w {:.2},{:.2}", wpos.x, wpos.y,))
            .with_color(colors::rgba(200, 200, 200, 220));
    }

    let color = colors::rgba(255, 255, 255, 150);
    let from_horiz = Vec2f::from(v2!(-win_w / 2, pos.y - win_h / 2));
    let to_horiz = Vec2f::from(v2!(win_w / 2, pos.y - win_h / 2));
    let from_vert = Vec2f::from(v2!(pos.x - win_w / 2, -win_h / 2));
    let to_vert = Vec2f::from(v2!(pos.x - win_w / 2, win_h / 2));
    painter.add_line(
        Line {
            from: from_horiz,
            to: to_horiz,
            thickness: 1.0,
        },
        color,
    );
    painter.add_line(
        Line {
            from: from_vert,
            to: to_vert,
            thickness: 1.0,
        },
        color,
    );
}

#[cfg(debug_assertions)]
fn update_win_debug_overlay(
    debug_overlay: &mut inle_debug::overlay::Debug_Overlay,
    window: &Render_Window_Handle,
) {
    let tsize = window::get_window_target_size(window);
    let rsize = window::get_window_real_size(window);
    debug_overlay.clear();
    debug_overlay
        .add_line(&format!(
            "WinSize: target = {}x{}, real = {}x{}",
            tsize.0, tsize.1, rsize.0, rsize.1
        ))
        .with_color(colors::rgba(110, 190, 250, 220));
}

#[cfg(debug_assertions)]
fn update_entities_debug_overlay(
    debug_overlay: &mut inle_debug::overlay::Debug_Overlay,
    ecs_world: &Ecs_World,
) {
    debug_overlay.clear();
    debug_overlay
        .add_line(&format!("Entities: {}", ecs_world.entities().len()))
        .with_color(colors::rgba(220, 100, 180, 220));
}

#[cfg(debug_assertions)]
fn update_camera_debug_overlay(
    debug_overlay: &mut inle_debug::overlay::Debug_Overlay,
    camera: &Transform2D,
) {
    debug_overlay.clear();
    debug_overlay
        .add_line(&format!(
            "[cam] pos: {:.2},{:.2}, scale: {:.1}",
            camera.position().x,
            camera.position().y,
            camera.scale().x
        ))
        .with_color(colors::rgba(220, 180, 100, 220));
}

#[cfg(debug_assertions)]
fn update_physics_debug_overlay(
    debug_overlay: &mut inle_debug::overlay::Debug_Overlay,
    collision_data: &physics::Collision_System_Debug_Data,
    chunks: &crate::spatial::World_Chunks,
) {
    debug_overlay.clear();
    debug_overlay
        .add_line(&format!(
            "[phys] n_inter_tests: {}, n_chunks: {}",
            collision_data.n_intersection_tests,
            chunks.n_chunks(),
        ))
        .with_color(colors::rgba(0, 173, 90, 220));
}

#[cfg(debug_assertions)]
fn update_record_debug_overlay(
    debug_overlay: &mut inle_debug::overlay::Debug_Overlay,
    recording: bool,
    replaying: bool,
) {
    debug_overlay.clear();
    if replaying {
        debug_overlay
            .add_line("REPLAYING")
            .with_color(colors::rgb(30, 200, 30));
    } else if recording {
        debug_overlay
            .add_line("RECORDING")
            .with_color(colors::rgb(200, 30, 30));
    }
}

#[cfg(debug_assertions)]
fn debug_draw_colliders(
    debug_painter: &mut Debug_Painter,
    ecs_world: &Ecs_World,
    phys_world: &Physics_World,
) {
    use crate::collisions::Game_Collision_Layer;
    use inle_physics::collider::{C_Collider, Collision_Shape};
    use std::convert::TryFrom;

    foreach_entity!(ecs_world,
        read: C_Collider, C_Spatial2D;
        write: ;
    |_e, (collider_comp, _spatial): (&C_Collider, &C_Spatial2D), ()| {
        for collider in phys_world.get_all_colliders(collider_comp.phys_body_handle) {
            // Note: since our collision detector doesn't handle rotation, draw the colliders with rot = 0
            // @Incomplete: scale?
            let mut transform = Transform2D::from_pos_rot_scale(collider.position, rad(0.), v2!(1., 1.));

            debug_painter.add_text(
                &Game_Collision_Layer::try_from(collider.layer).map_or_else(
                    |_| format!("? {}", collider.layer),
                    |gcl| format!("{:?}", gcl),
                ),
                collider.position,
                5,
                colors::BLACK);

            let mut cld_color = colors::rgba(255, 255, 0, 100);

            let colliding_with = phys_world.get_collisions(collider.handle);
            if !colliding_with.is_empty() {
                cld_color = colors::rgba(255, 0, 0, 100);
            }

            for cls_data in colliding_with {
                let oth_cld = phys_world.get_collider(cls_data.other_collider).unwrap();
                debug_painter.add_arrow(Arrow {
                    center: collider.position,
                    direction: oth_cld.position - collider.position,
                    thickness: 1.,
                    arrow_size: 5.,
                }, colors::GREEN);
                debug_painter.add_arrow(Arrow {
                    center: collider.position, // @Incomplete: it'd be nice to have the exact collision position
                    direction: cls_data.info.normal * 20.0,
                    thickness: 1.,
                    arrow_size: 5.,
                }, colors::PINK);
            }

            match collider.shape {
                Collision_Shape::Rect { width, height } => {
                    transform.translate(-width * 0.5, -height * 0.5);
                    debug_painter.add_rect(Vec2f::new(width, height), &transform, cld_color);
                }
                Collision_Shape::Circle { radius } => {
                    transform.translate(-radius * 0.5, -radius * 0.5);
                    debug_painter.add_circle(
                        Circle {
                            center: collider.position,
                            radius,
                        },
                        cld_color,
                    );
                }
                _ => {}
            }

            debug_painter.add_circle(
                Circle {
                    center: collider.position,
                    radius: 2.,
                },
                colors::ORANGE);

            debug_painter.add_text(
                &format!("{},{}", collider.handle.gen, collider.handle.index),
                collider.position + v2!(2., -3.),
                5, colors::ORANGE);
        }
    });
}

#[cfg(debug_assertions)]
fn debug_draw_transforms(
    debug_painter: &mut Debug_Painter,
    ecs_world: &Ecs_World,
    window: &Render_Window_Handle,
    input_state: &Input_State,
    camera: &Transform2D,
) {
    use inle_ecs::ecs_world::Entity;

    let mpos = render_window::mouse_pos_in_world(window, &input_state.raw.mouse_state, camera);
    let mut entity_overlapped = (Entity::INVALID, 0.);
    foreach_entity!(ecs_world,
        read: C_Spatial2D;
        write: ;
        |entity, (spatial, ): (&C_Spatial2D,), ()| {
        let transform = &spatial.transform;
        let center = transform.position();
        let radius = 5.0;
        debug_painter.add_circle(
            Circle {
                radius,
                center,
            },
            colors::rgb(50, 100, 200),
        );

        let overlap_radius = 8.0 * radius;
        let dist2 = (center - mpos).magnitude2();
        let overlaps = dist2 < overlap_radius * overlap_radius;
        if overlaps && (entity_overlapped.0 == Entity::INVALID || dist2 < entity_overlapped.1) {
            entity_overlapped = (entity, dist2);
        }

        debug_painter.add_text(
            &format!(
                "{:.2},{:.2}",
                transform.position().x,
                transform.position().y
            ),
            transform.position(),
            7,
            Paint_Properties {
                color: colors::WHITE,
                border_thick: 1.,
                border_color: colors::BLACK,
                ..Default::default()
            },
        );
    });

    if entity_overlapped.0 != Entity::INVALID {
        let spatial = ecs_world
            .get_component::<C_Spatial2D>(entity_overlapped.0)
            .unwrap();
        debug_painter.add_text(
            &format!("{:?}", entity_overlapped.0),
            spatial.transform.position() - v2!(0., 10.),
            7,
            Paint_Properties {
                color: colors::WHITE,
                border_thick: 1.,
                border_color: colors::BLACK,
                ..Default::default()
            },
        );
    }
}

#[cfg(debug_assertions)]
fn debug_draw_velocities(debug_painter: &mut Debug_Painter, ecs_world: &Ecs_World) {
    const COLOR: colors::Color = colors::rgb(100, 0, 120);

    foreach_entity!(ecs_world,
        read: C_Spatial2D;
        write: ;
    |_e, (spatial, ): (&C_Spatial2D, ), ()| {
        if spatial.velocity.magnitude2() > 0. {
            let transform = &spatial.transform;
            debug_painter.add_arrow(
                Arrow {
                    center: transform.position(),
                    direction: spatial.velocity * 0.5,
                    thickness: 3.,
                    arrow_size: 20.,
                },
                COLOR,
            );
            debug_painter.add_shaded_text(
                &spatial.velocity.to_string(),
                transform.position() + Vec2f::new(1., -15.),
                12,
                COLOR,
                colors::WHITE,
            );
        }
    });
}

#[cfg(debug_assertions)]
fn debug_draw_component_lists(debug_painter: &mut Debug_Painter, ecs_world: &Ecs_World) {
    use crate::debug::entity_debug::C_Debug_Data;

    foreach_entity!(ecs_world,
        read: ;
        write: ;
    |entity, (), ()| {
        let pos = if let Some(spatial) = ecs_world.get_component::<C_Spatial2D>(entity) {
            spatial.transform.position() + v2!(0., -15.)
        } else {
            v2!(0., 0.) // @Incomplete
        };

         if let Some(debug) = ecs_world.get_component::<C_Debug_Data>(entity) {
            let name = debug.entity_name.as_ref();
            debug_painter.add_shaded_text(name, pos, 7, colors::GREEN, colors::BLACK);
        } else {
            debug_painter.add_shaded_text("<Unknown>", pos, 7, colors::GREEN, colors::BLACK);
        }

        for (i, comp_name) in ecs_world.get_comp_name_list_for_entity(entity).iter().enumerate() {
            debug_painter.add_shaded_text_with_shade_distance(
                &format!(
                    " {}", comp_name
                ),
                pos + v2!(0., (i + 1) as f32 * 8.5),
                6,
                colors::WHITE,
                colors::BLACK,
                v2!(0.5, 0.5),
            );
        }
    });
}

#[cfg(debug_assertions)]
fn debug_draw_lights(
    screenspace_debug_painter: &mut Debug_Painter,
    debug_painter: &mut Debug_Painter,
    lights: &inle_gfx::light::Lights,
) {
    screenspace_debug_painter.add_shaded_text(
        &format!(
            "Ambient Light: color: #{:X}, intensity: {}",
            colors::color_to_hex_no_alpha(lights.ambient_light().color),
            lights.ambient_light().intensity
        ),
        v2!(5., 300.),
        15,
        lights.ambient_light().color,
        if colors::to_hsv(lights.ambient_light().color).v > 0.5 {
            colors::BLACK
        } else {
            colors::WHITE
        },
    );
    for pl in lights.point_lights() {
        debug_painter.add_circle(
            Circle {
                center: pl.position,
                radius: pl.radius,
            },
            Paint_Properties {
                color: colors::TRANSPARENT,
                border_color: pl.color,
                border_thick: 2.,
                ..Default::default()
            },
        );
        debug_painter.add_circle(
            Circle {
                center: pl.position,
                radius: 2.,
            },
            pl.color,
        );
        debug_painter.add_shaded_text(
            &format!(
                "radius: {}\natten: {}\nintens: {}",
                pl.radius, pl.attenuation, pl.intensity
            ),
            pl.position,
            10,
            pl.color,
            if colors::to_hsv(pl.color).v > 0.5 {
                colors::BLACK
            } else {
                colors::WHITE
            },
        );
    }

    for rl in lights.rect_lights() {
        debug_painter.add_rect(
            rl.rect.size(),
            &Transform2D::from_pos(rl.rect.pos_min()),
            colors::rgba(rl.color.r, rl.color.g, rl.color.b, 50),
        );
        debug_painter.add_rect(
            rl.rect.size(),
            &Transform2D::from_pos(rl.rect.pos_min()),
            Paint_Properties {
                color: colors::TRANSPARENT,
                border_color: colors::WHITE,
                border_thick: 1.,
                ..Default::default()
            },
        );
        // @Incomplete: we should actually add a capsule...add support for it in the painter!
        debug_painter.add_rect(
            v2!(
                rl.rect.width + 2. * rl.radius,
                rl.rect.height + 2. * rl.radius
            ),
            &Transform2D::from_pos(rl.rect.pos_min() - v2!(rl.radius, rl.radius)),
            Paint_Properties {
                color: colors::TRANSPARENT,
                border_color: rl.color,
                border_thick: 1.,
                ..Default::default()
            },
        );
        debug_painter.add_shaded_text(
            &format!(
                "radius: {}\natten: {}\nintens: {}",
                rl.radius, rl.attenuation, rl.intensity
            ),
            rl.rect.pos_max(),
            10,
            rl.color,
            if colors::to_hsv(rl.color).v > 0.5 {
                colors::BLACK
            } else {
                colors::WHITE
            },
        );
    }
}

#[cfg(debug_assertions)]
fn debug_draw_entities_prev_frame_ghost(
    window: &mut Render_Window_Handle,
    batches: &mut inle_gfx::render::batcher::Batches,
    shader_cache: &mut inle_resources::gfx::Shader_Cache,
    env: &inle_core::env::Env_Info,
    ecs_world: &mut Ecs_World,
    is_paused: bool,
) {
    trace!("debug_draw_entities_prev_frame_ghost");

    use crate::debug::entity_debug::C_Debug_Data;
    //use crate::systems::pixel_collision_system::C_Texture_Collider;
    use crate::gfx::shaders::SHD_SPRITE_UNLIT;
    use inle_gfx::components::C_Renderable;
    use inle_gfx::render;
    use inle_resources::gfx::shader_path;

    let unlit_shader = shader_cache.load_shader(&shader_path(env, SHD_SPRITE_UNLIT));

    foreach_entity!(ecs_world,
        read: C_Spatial2D, C_Renderable;
        write:  C_Debug_Data;
        |_e, (spatial, renderable): (&C_Spatial2D, &C_Renderable), (debug_data,): (&mut C_Debug_Data,)| {
        let frame_starting_pos = spatial.frame_starting_pos;
        let C_Renderable {
            material,
            rect,
            modulate,
            z_index,
            sprite_local_transform,
        } = *renderable;

        let mut material = material;
        material.cast_shadows = false;
        material.shader = unlit_shader;

        if !is_paused {
            if (debug_data.n_prev_positions_filled as usize) < debug_data.prev_positions.len() {
                debug_data.prev_positions[debug_data.n_prev_positions_filled as usize] = frame_starting_pos;
                debug_data.n_prev_positions_filled += 1;
            } else {
                for i in 0..debug_data.prev_positions.len() - 1 {
                    debug_data.prev_positions[i] = debug_data.prev_positions[i + 1];
                }
                debug_data.prev_positions[debug_data.prev_positions.len() - 1] = frame_starting_pos;
            }
        }

        for i in 0..debug_data.n_prev_positions_filled {
            let transform = Transform2D::from_pos(debug_data.prev_positions[i as usize]);
            let color = colors::rgba(
                modulate.r,
                modulate.g,
                modulate.b,
                200 - 10 * (debug_data.prev_positions.len() - i as usize) as u8,
            );
            render::render_texture_ws(window, batches, &material, &rect, color, &transform.combine(&sprite_local_transform), z_index);
        }
    });
}

#[cfg(debug_assertions)]
fn debug_draw_entities_pos_history(painter: &mut Debug_Painter, ecs_world: &Ecs_World) {
    use crate::debug::systems::position_history_system::C_Position_History;
    use inle_math::math::lerp;

    foreach_entity!(ecs_world,
        read: C_Position_History;
        write: ;
        |_e, (pos_hist,): (&C_Position_History,), ()| {
        let positions = &pos_hist.positions;
        let mut last_pos_latest_slice = None;
        let mut idx = 0;
        let (slice1, slice2) = positions.as_slices();
        for slice in &[slice1, slice2] {
            if slice.is_empty() {
                continue;
            }
            if let Some(prev_pos) = last_pos_latest_slice {
                let t = idx as f32 / positions.len() as f32;
                painter.add_line(
                    Line {
                        from: prev_pos,
                        to: slice[0],
                        thickness: lerp(0.5, 1.5, t),
                    },
                    colors::rgba(255, 255, 255, lerp(30.0, 225.0, t) as u8),
                );
            }
            for pair in slice.windows(2) {
                let prev_pos = pair[0];
                let pos = pair[1];
                let t = idx as f32 / positions.len() as f32;
                painter.add_line(
                    Line {
                        from: prev_pos,
                        to: pos,
                        thickness: lerp(0.5, 1.5, t),
                    },
                    colors::rgba(255, 255, 255, lerp(30.0, 225.0, t) as u8),
                );
                idx += 1;
            }
            last_pos_latest_slice = Some(slice[slice.len() - 1]);
        }
    });
}

/// Draws a grid made of squares, each of size `square_size`.
#[cfg(debug_assertions)]
fn debug_draw_grid(
    debug_painter: &mut Debug_Painter,
    camera_transform: &Transform2D,
    (screen_width, screen_height): (u32, u32),
    square_size: f32,
    grid_opacity: u8,
    font_size: u16,
) {
    let Vec2f { x: cx, y: cy } = camera_transform.position();
    let Vec2f {
        x: cam_sx,
        y: cam_sy,
    } = camera_transform.scale();
    let sw = screen_width as f32 * cam_sx;
    let sh = screen_height as f32 * cam_sy;
    let n_horiz = (sw / square_size).floor() as usize + 2;
    let n_vert = (sh / square_size).floor() as usize + 2;
    if n_vert * n_horiz > 14_000 {
        return; // let's not kill the machine if we can help
    }

    let col_gray = colors::rgba(200, 200, 200, grid_opacity);
    let col_white = colors::rgba(255, 255, 255, grid_opacity);
    let sq_coord = Vec2f::new(
        ((cx * cam_sx - sw * 0.5) / square_size).floor() * square_size,
        ((cy * cam_sy - sh * 0.5) / square_size).floor() * square_size,
    );

    let draw_text = n_vert * n_horiz < 1000;
    for j in 0..n_vert {
        for i in 0..n_horiz {
            let transf = Transform2D::from_pos(
                sq_coord + Vec2f::new(i as f32 * square_size, j as f32 * square_size),
            );
            let color = if ((i as i32 - (sq_coord.x / square_size) as i32)
                + (j as i32 - (sq_coord.y / square_size) as i32))
                % 2
                == 0
            {
                col_white
            } else {
                col_gray
            };
            let pos = transf.position();
            debug_painter.add_rect(Vec2f::new(square_size, square_size), &transf, color);
            if draw_text {
                debug_painter.add_text(
                    &format!("{},{}", pos.x, pos.y),
                    pos + Vec2f::new(5., 5.),
                    font_size,
                    color,
                );
            }
        }
    }
}

#[cfg(debug_assertions)]
fn update_graph_fps(
    graph: &mut inle_debug::graph::Debug_Graph_View,
    time: &time::Time,
    fps: &inle_debug::fps::Fps_Counter,
) {
    const TIME_LIMIT: f32 = 60.0;

    let fps = fps.get_instant_fps();
    inle_debug::graph::add_point_and_scroll(graph, time.real_time(), TIME_LIMIT, fps);
}

#[cfg(debug_assertions)]
fn update_graph_prev_frame_t(
    graph: &mut inle_debug::graph::Debug_Graph_View,
    time: &time::Time,
    prev_frame_t: &Duration,
) {
    const TIME_LIMIT: f32 = 10.0;

    inle_debug::graph::add_point_and_scroll(
        graph,
        time.real_time(),
        TIME_LIMIT,
        prev_frame_t.as_secs_f32() * 1000.,
    );
}

#[cfg(debug_assertions)]
fn get_render_system_debug_visualization(
    debug_cvars: &super::game_state::Debug_CVars,
    cfg: &inle_cfg::Config,
) -> render_system::Debug_Visualization {
    match debug_cvars.render_debug_visualization.read(cfg).as_str() {
        "1" | "b" | "bounds" => render_system::Debug_Visualization::Sprites_Boundaries,
        "2" | "n" | "normals" => render_system::Debug_Visualization::Normals,
        "3" | "m" | "materials" => render_system::Debug_Visualization::Materials,
        _ => render_system::Debug_Visualization::None,
    }
}

#[cfg(debug_assertions)]
fn set_debug_hud_enabled(debug_ui: &mut inle_debug::debug_ui::Debug_Ui_System, enabled: bool) {
    debug_ui.set_overlay_enabled(sid!("time"), enabled);
    debug_ui.set_overlay_enabled(sid!("fps"), enabled);
    debug_ui.set_overlay_enabled(sid!("window"), enabled);
    debug_ui.set_overlay_enabled(sid!("entities"), enabled);
    debug_ui.set_overlay_enabled(sid!("camera"), enabled);
    debug_ui.set_overlay_enabled(sid!("physics"), enabled);
    debug_ui.set_overlay_enabled(sid!("joysticks"), enabled);
    debug_ui.frame_scroller.hidden = !enabled;
}

#[cfg(debug_assertions)]
fn debug_draw_particle_emitters(
    painter: &mut Debug_Painter,
    particle_mgr: &inle_gfx::particles::Particle_Manager,
) {
    for emitter in &particle_mgr.active_emitters {
        let aabb = inle_math::rect::aabb_of_transformed_rect(&emitter.bounds, &emitter.transform);
        painter.add_rect(
            aabb.size(),
            &Transform2D::from_pos(aabb.pos_min()),
            Paint_Properties {
                color: colors::rgba(58, 162, 252, 80),
                border_color: colors::rgb(58, 162, 252),
                border_thick: 1.0,
                ..Default::default()
            },
        );
        painter.add_rect(
            emitter.bounds.size(),
            &emitter
                .transform
                .combine(&Transform2D::from_pos(emitter.bounds.pos_min())),
            Paint_Properties {
                color: colors::rgba(208, 80, 232, 100),
                border_color: colors::rgb(208, 80, 232),
                border_thick: 1.0,
                ..Default::default()
            },
        );
        painter.add_arrow(
            Arrow {
                center: emitter.transform.position(),
                direction: Vec2f::from_rotation(emitter.transform.rotation()) * 30.0,
                thickness: 1.0,
                arrow_size: 20.,
            },
            colors::rgb(0, 141, 94),
        );
    }
}
