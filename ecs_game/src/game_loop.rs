use super::{Game_Resources, Game_State};
use crate::states::state::Game_State_Args;
use inle_app::{app, render_system};
use inle_common::colors;
use inle_core::time;
use inle_math::transform::Transform2D;
use inle_physics::physics;
use inle_resources::gfx::Gfx_Resources;
use std::time::Duration;

#[cfg(debug_assertions)]
use {
    crate::debug::debug_funcs::*, inle_common::stringid::String_Id, inle_win::window,
    std::collections::HashMap,
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

            game_state.gameplay_system.realtime_update(
                &real_dt,
                &game_state.window,
                &mut game_state.engine_state,
            );

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

                    let render_system = &mut game_state.engine_state.systems.render_system;
                    #[cfg(debug_assertions)]
                    let debug_queries = &mut game_state.debug_ecs_queries;
                    #[cfg(debug_assertions)]
                    let game_debug_systems = &mut game_state.game_debug_systems;
                    game_state.gameplay_system.update_pending_component_updates(
                        |pending_updates, comp_mgr| {
                            render_system.update_queries(pending_updates, comp_mgr);
                            #[cfg(debug_assertions)]
                            {
                                debug_queries.update_queries(pending_updates, comp_mgr);
                                game_debug_systems.update_queries(pending_updates, comp_mgr);
                            }
                        },
                    );

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
        let render_system = &mut game_state.engine_state.systems.render_system;
        let batches = &mut game_state.level_batches;
        let frame_alloc = &mut game_state.engine_state.frame_alloc;
        #[cfg(debug_assertions)]
        let painters = &mut game_state.engine_state.debug_systems.painters;
        gameplay_system.levels.foreach_active_level(|level| {
            let render_args = render_system::Render_System_Update_Args {
                batches: batches.get_mut(&level.id).unwrap(),
                ecs_world: &level.world,
                frame_alloc,
                render_cfg,
                window,
                camera: &level.get_camera_transform(),
                gres,
                shader_cache,
                #[cfg(debug_assertions)]
                painter: painters.get_mut(&level.id).unwrap(),
            };

            render_system.update(render_args);
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
    let frame_alloc = &mut game_state.engine_state.frame_alloc;
    let phys_settings = &game_state.engine_state.systems.physics_settings;
    let evt_register = &mut game_state.engine_state.systems.evt_register;

    gameplay_system.pre_update_physics();

    gameplay_system.levels.foreach_active_level(|level| {
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
    });

    gameplay_system.update_physics(update_dt, frame_alloc);
}

#[cfg(debug_assertions)]
fn update_debug(
    game_state: &mut Game_State,
    game_resources: &mut Game_Resources,
    collision_debug_data: HashMap<String_Id, physics::Collision_System_Debug_Data>,
    dt: Duration,
) {
    let display_overlays = game_state
        .debug_cvars
        .display_overlays
        .read(&game_state.engine_state.config);

    draw_debug_overlays(game_state, display_overlays);

    {
        let engine_state = &mut game_state.engine_state;
        let debug_systems = &mut engine_state.debug_systems;

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
    }

    draw_debug_graphs(game_state);

    ////// Per-Level debugs //////
    let input_cfg = &game_state.gameplay_system.input_cfg;
    let game_debug_systems = &game_state.game_debug_systems;
    let engine_state = &mut game_state.engine_state;
    game_state
        .gameplay_system
        .levels
        .foreach_active_level(|level| {
            let mut update_args = crate::systems::interface::Update_Args {
                dt,
                ecs_world: &mut level.world,
                phys_world: &mut level.phys_world,
                engine_state,
                input_cfg,
            };
            game_debug_systems.update(&mut update_args);
        });

    let debug_systems = &mut engine_state.debug_systems;
    let painters = &mut debug_systems.painters;
    let debug_ui = &mut debug_systems.debug_ui;
    let target_win_size = engine_state.app_config.target_win_size;

    let is_paused = engine_state.time.paused && !engine_state.time.is_stepping();
    let cvars = &game_state.debug_cvars;
    let cfg = &engine_state.config;
    let draw_entities = cvars.draw_entities.read(cfg);
    let draw_component_lists = cvars.draw_component_lists.read(cfg);
    let draw_velocities = cvars.draw_velocities.read(cfg);
    let draw_entity_prev_frame_ghost = cvars.draw_entity_prev_frame_ghost.read(cfg);
    let draw_entity_pos_history = cvars.draw_entity_pos_history.read(cfg);
    let draw_colliders = cvars.draw_colliders.read(cfg);
    let draw_debug_grid = cvars.draw_debug_grid.read(cfg);
    let grid_square_size = cvars.debug_grid_square_size.read(cfg);
    let grid_font_size = cvars.debug_grid_font_size.read(cfg);
    let grid_opacity = cvars.debug_grid_opacity.read(cfg) as u8;
    let draw_world_chunks = cvars.draw_world_chunks.read(cfg);
    let draw_lights = cvars.draw_lights.read(cfg);
    let draw_particle_emitters = cvars.draw_particle_emitters.read(cfg);
    let draw_entities_touching_ground = cvars.draw_entities_touching_ground.read(cfg);
    let lv_batches = &mut game_state.level_batches;
    let global_painter = &mut debug_systems.global_painter;
    let window = &mut game_state.window;
    let shader_cache = &mut game_resources.shader_cache;
    let env = &engine_state.env;
    let particle_mgrs = &engine_state.systems.particle_mgrs;
    let queries = &game_state.debug_ecs_queries;
    let input_state = &engine_state.input_state;

    game_state
        .gameplay_system
        .levels
        .foreach_active_level(|level| {
            let debug_painter = painters
                .get_mut(&level.id)
                .unwrap_or_else(|| fatal!("Debug painter not found for level {:?}", level.id));

            if display_overlays {
                update_entities_and_draw_calls_debug_overlay(
                    debug_ui.get_overlay(sid!("entities")),
                    &level.world,
                    window,
                );
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
                    &queries.just_spatials,
                    &level.world,
                    window,
                    input_state,
                    &level.get_camera_transform(),
                );
            }

            if draw_velocities {
                debug_draw_velocities(debug_painter, &queries.just_spatials, &level.world);
            }

            if draw_entity_prev_frame_ghost {
                let batches = lv_batches.get_mut(&level.id).unwrap();
                debug_draw_entities_prev_frame_ghost(
                    window,
                    batches,
                    shader_cache,
                    env,
                    &queries.draw_entities_prev_frame_ghost,
                    &mut level.world,
                    is_paused,
                );
            }

            if draw_entity_pos_history {
                debug_draw_entities_pos_history(
                    debug_painter,
                    &queries.draw_entities_pos_history,
                    &level.world,
                );
            }

            if draw_entities_touching_ground {
                debug_draw_entities_touching_ground(
                    debug_painter,
                    &queries.draw_entities_touching_ground,
                    &level.world,
                );
            }

            if draw_component_lists {
                debug_draw_component_lists(
                    debug_painter,
                    &queries.draw_component_lists,
                    &level.world,
                );
            }

            if draw_colliders {
                debug_draw_colliders(
                    debug_painter,
                    &queries.draw_colliders,
                    &level.world,
                    &level.phys_world,
                );
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
                level.chunks.debug_draw(debug_painter, &level.phys_world);
            }
        });

    // @Cleanup
    if cvars.draw_buf_alloc.read(&engine_state.config) {
        inle_debug::backend_specific_debugs::draw_backend_specific_debug(window, global_painter);
    }
}

#[cfg(debug_assertions)]
fn draw_debug_overlays(game_state: &mut Game_State, display_overlays: bool) {
    let engine_state = &mut game_state.engine_state;
    let debug_systems = &mut engine_state.debug_systems;

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
            &engine_state.prev_frame_time,
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
}

#[cfg(debug_assertions)]
fn draw_debug_graphs(game_state: &mut Game_State) {
    let engine_state = &mut game_state.engine_state;
    let debug_systems = &mut engine_state.debug_systems;

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
}
