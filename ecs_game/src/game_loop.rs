use super::{Game_Resources, Game_State};
use crate::states::state::Game_State_Args;
use ecs_engine::alloc::temp::*;
use ecs_engine::collisions::physics;
use ecs_engine::common::colors;
use ecs_engine::common::transform::Transform2D;
use ecs_engine::core::app;
use ecs_engine::core::time;
use ecs_engine::gfx;
use ecs_engine::gfx::render_system;
use ecs_engine::gfx::render_window::Render_Window_Handle;
use ecs_engine::input;
use ecs_engine::resources::gfx::Gfx_Resources;
use ecs_engine::ui;
use std::convert::TryInto;
use std::time::Duration;

#[cfg(debug_assertions)]
use {
    ecs_engine::{
        common::angle::rad,
        common::shapes::{Arrow, Circle},
        common::stringid::String_Id,
        common::vector::Vec2f,
        debug,
        debug::painter::Debug_Painter,
        ecs::components::base::C_Spatial2D,
        ecs::ecs_world::{self, Ecs_World},
        gfx::paint_props::Paint_Properties,
        gfx::window,
    },
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
    game_state.engine_state.time.update();
    let dt = game_state.engine_state.time.dt();
    let real_dt = game_state.engine_state.time.real_dt();

    let target_time_per_frame = Duration::from_micros(
        (game_state
            .cvars
            .gameplay_update_tick_ms
            .read(&game_state.engine_state.config)
            * 1000.0) as u64,
    );

    #[cfg(debug_assertions)]
    let debug_systems = &mut game_state.engine_state.debug_systems;

    let mut_in_debug!(replaying) = false;

    // Update input
    {
        trace!("input_state::update");

        let mut_in_debug!(process_game_actions) = true;

        #[cfg(debug_assertions)]
        {
            process_game_actions =
                debug_systems.console.status != debug::console::Console_Status::Open;
        }

        // Update the raw input. This may be overwritten later by replay data, but its core_events won't.
        input::input_state::update_raw_input(
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
                        .extend(&raw_input.core_events);
                    game_state.engine_state.input_state.raw.core_events = raw_input.core_events;
                } else {
                    game_state.engine_state.input_state.raw.events =
                        game_state.engine_state.input_state.raw.core_events.clone();
                }
            } else {
                game_state.engine_state.replay_input_provider = None;
            }
        }

        input::input_state::process_raw_input(
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
        if debug_systems.console.status == debug::console::Console_Status::Open {
            debug_systems
                .console
                .update(&game_state.engine_state.input_state.raw.events);

            let mut output = vec![];
            while let Some(cmd) = game_state
                .engine_state
                .debug_systems
                .console
                .pop_enqueued_cmd()
            {
                if !cmd.is_empty() {
                    let maybe_output = console_executor::execute(
                        &cmd,
                        &mut game_state.engine_state,
                        &mut game_state.gameplay_system,
                    );
                    if let Some(out) = maybe_output {
                        output.push(out);
                    }
                }
            }

            for (out, color) in output {
                game_state
                    .engine_state
                    .debug_systems
                    .console
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
        ) {
            game_state.engine_state.should_close = true;
            return Ok(());
        }
    }

    {
        #[cfg(debug_assertions)]
        {
            let input_state = &game_state.engine_state.input_state;

            update_joystick_debug_overlay(
                debug_systems
                    .debug_ui
                    .get_overlay(String_Id::from("joysticks")),
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
                debug_systems
                    .debug_ui
                    .get_overlay(String_Id::from("record")),
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
            };
            game_state.state_mgr.handle_actions(&actions, &mut args);
            if game_state.engine_state.should_close {
                return Ok(());
            }
        }

        #[cfg(debug_assertions)]
        {
            use ecs_engine::debug::console::Console_Status;
            use ecs_engine::input::input_state::Action_Kind;

            let actions = &game_state.engine_state.input_state.processed.game_actions;

            if actions.contains(&(String_Id::from("toggle_console"), Action_Kind::Pressed)) {
                game_state.engine_state.debug_systems.console.toggle();
            }

            if actions.contains(&(String_Id::from("calipers"), Action_Kind::Pressed)) {
                if let Some(level) = game_state.gameplay_system.levels.first_active_level() {
                    game_state
                        .engine_state
                        .debug_systems
                        .calipers
                        .start_measuring_dist(&game_state.window, &level.get_camera().transform);
                }
            } else if actions.contains(&(String_Id::from("calipers"), Action_Kind::Released)) {
                game_state
                    .engine_state
                    .debug_systems
                    .calipers
                    .end_measuring();
            }

            window::set_key_repeat_enabled(
                &mut game_state.window,
                game_state.engine_state.debug_systems.console.status == Console_Status::Open,
            );
        }

        // Update game systems
        {
            trace!("game_update");

            game_state.gameplay_system.realtime_update(
                &real_dt,
                &game_state.engine_state.input_state,
                &game_state.engine_state.config,
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
                };
                game_state.state_mgr.update(&mut args, &dt, &real_dt);
            }

            let time = &game_state.engine_state.time;
            // @Cleanup @Soundness: either pass to the update the "actual" dt or accumulate the extra time to
            // have a really fixed time step. That depends if we care about being deterministic or not.
            let update_dt = time::mul_duration(
                &target_time_per_frame,
                time.time_scale * (!time.paused as u32 as f32),
            );
            game_state.gameplay_system.update(
                &update_dt,
                &mut game_state.engine_state,
                game_resources,
                &game_state.window,
            );
        }
    }

    #[cfg(debug_assertions)]
    let mut collision_debug_data = HashMap::new();

    // Update collisions
    {
        trace!("physics::update");

        let gameplay_system = &mut game_state.gameplay_system;
        let time = &game_state.engine_state.time;
        let update_dt = time::mul_duration(
            &target_time_per_frame,
            time.time_scale * (!time.paused as u32 as f32),
        );
        let pixel_collision_system = &mut gameplay_system.pixel_collision_system;
        let levels = &gameplay_system.levels;
        let frame_alloc = &mut game_state.engine_state.frame_alloc;
        let phys_settings = &game_state.engine_state.systems.physics_settings;
        #[cfg(debug_assertions)]
        let debug_systems = &mut game_state.engine_state.debug_systems;

        levels.foreach_active_level(|level| {
            #[cfg(debug_assertions)]
            let coll_debug = collision_debug_data
                .entry(level.id)
                .or_insert_with(physics::Collision_System_Debug_Data::default);

            physics::update_collisions(
                &mut level.world,
                &level.chunks,
                phys_settings,
                #[cfg(debug_assertions)]
                coll_debug,
            );

            pixel_collision_system.update(
                &mut level.world,
                &game_resources.gfx,
                &phys_settings.collision_matrix,
                frame_alloc,
                #[cfg(debug_assertions)]
                debug_systems.painters.get_mut(&level.id).unwrap(),
            );

            {
                let mut moved = excl_temp_array(frame_alloc);
                crate::movement_system::update(&update_dt, &mut level.world, &mut moved);

                let moved = unsafe { moved.into_read_only() };
                for mov in &moved {
                    level.chunks.update_entity(
                        mov.entity,
                        mov.prev_pos,
                        mov.new_pos,
                        mov.extent,
                        frame_alloc,
                    );
                }
            }
        });
    }

    game_state
        .gameplay_system
        .late_update(&mut game_state.engine_state.systems.evt_register);

    // Update audio
    {
        trace!("audio_system_update");
        game_state.engine_state.systems.audio_system.update();
    }

    // We clear batches before update_debug, so debug can draw textures
    {
        trace!("clear_batches");
        gfx::render::batcher::clear_batches(&mut game_state.engine_state.global_batches);
        for batches in game_state.level_batches.values_mut() {
            gfx::render::batcher::clear_batches(batches);
        }
    }

    #[cfg(debug_assertions)]
    update_debug(game_state, collision_debug_data);

    update_graphics(game_state, &mut game_resources.gfx);
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
        gfx::window::display(&mut game_state.window);
    }

    Ok(())
}

fn update_ui(game_state: &mut Game_State, gres: &Gfx_Resources) {
    trace!("update_ui");

    let window = &mut game_state.window;
    let ui_ctx = &mut game_state.engine_state.systems.ui;

    ui::draw_all_ui(window, gres, ui_ctx);
}

fn update_graphics<'a, 's, 'r>(game_state: &'a mut Game_State<'s>, gres: &'a mut Gfx_Resources<'r>)
where
    'r: 's,
    's: 'a,
{
    trace!("update_graphics");

    let window = &mut game_state.window;

    #[cfg(debug_assertions)]
    {
        let cur_vsync = gfx::window::has_vsync(window);
        let desired_vsync = game_state.cvars.vsync.read(&game_state.engine_state.config);
        if desired_vsync != cur_vsync {
            gfx::window::set_vsync(window, desired_vsync);
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
        gfx::render_window::set_clear_color(window, clear_color);
        gfx::render_window::clear(window);
    }

    let cfg = &game_state.engine_state.config;
    let render_cfg = render_system::Render_System_Config {
        clear_color: colors::color_from_hex(game_state.cvars.clear_color.read(cfg) as u32),
        #[cfg(debug_assertions)]
        draw_sprites_bg: game_state.debug_cvars.draw_sprites_bg.read(cfg),
        #[cfg(debug_assertions)]
        draw_sprites_bg_color: colors::color_from_hex(
            game_state.debug_cvars.draw_sprites_bg_color.read(cfg) as u32,
        ),
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
                camera: &level.get_camera().transform,
            };

            render_system::update(render_args);
        });
    }

    // Draw texture batches
    {
        let frame_alloc = &mut game_state.engine_state.frame_alloc;
        let lv_batches = &mut game_state.level_batches;
        let window = &mut game_state.window;
        let shader_cache = &mut game_state.engine_state.shader_cache;
        let enable_shaders = game_state.cvars.enable_shaders.read(cfg);

        game_state
            .gameplay_system
            .levels
            .foreach_active_level(|level| {
                gfx::render::batcher::draw_batches(
                    window,
                    &gres,
                    lv_batches.get_mut(&level.id).unwrap(),
                    shader_cache,
                    &level.get_camera().transform,
                    &level.lights,
                    enable_shaders,
                    frame_alloc,
                );
            });
        gfx::render::batcher::draw_batches(
            window,
            &gres,
            &mut game_state.engine_state.global_batches,
            shader_cache,
            &Transform2D::default(),
            &ecs_engine::gfx::light::Lights::default(),
            enable_shaders,
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
                &level.get_camera().transform,
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
            painter.draw(window, gres, &level.get_camera().transform);
            painter.clear();
        });

    // Global painter
    let painter = &mut game_state.engine_state.debug_systems.global_painter;
    painter.draw(&mut game_state.window, gres, &Transform2D::default());
    painter.clear();

    // Draw debug UI
    {
        trace!("debug_ui::update");
        let debug_ui = &mut game_state.engine_state.debug_systems.debug_ui;
        let prev_selected = debug_ui
            .get_graph(String_Id::from("fn_profile"))
            .get_selected_point();
        debug_ui.update_and_draw(
            &real_dt,
            &mut game_state.window,
            gres,
            &game_state.engine_state.input_state,
            &game_state.engine_state.debug_systems.log,
            &mut game_state.engine_state.frame_alloc,
        );

        let profile_graph = debug_ui.get_graph(String_Id::from("fn_profile"));
        let cur_selected = profile_graph.get_selected_point();
        if cur_selected != prev_selected {
            game_state.engine_state.time.paused = cur_selected.is_some();
            debug_ui.frame_scroller.manually_selected = cur_selected.is_some();
            if let Some(sel) = cur_selected {
                let profile_graph = debug_ui.get_graph(String_Id::from("fn_profile"));
                // @Robustness @Refactoring: this should be a u64
                let real_frame: u32 = profile_graph
                    .data
                    .get_point_metadata(sel.index, String_Id::from("real_frame"))
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
            .draw(&mut game_state.window, gres);
    }
}

#[cfg(debug_assertions)]
fn update_debug(
    game_state: &mut Game_State,
    collision_debug_data: HashMap<String_Id, physics::Collision_System_Debug_Data>,
) {
    let engine_state = &mut game_state.engine_state;
    let debug_systems = &mut engine_state.debug_systems;

    // @Speed @WaitForStable: these should all be computed at compile time.
    let (
        sid_time,
        sid_fps,
        sid_entities,
        sid_camera,
        sid_mouse,
        sid_window,
        sid_prev_frame_time,
        sid_physics,
    ) = (
        String_Id::from("time"),
        String_Id::from("fps"),
        String_Id::from("entities"),
        String_Id::from("camera"),
        String_Id::from("mouse"),
        String_Id::from("window"),
        String_Id::from("prev_frame_time"),
        String_Id::from("physics"),
    );

    // Overlays
    update_time_debug_overlay(
        debug_systems.debug_ui.get_overlay(sid_time),
        &engine_state.time,
    );

    update_fps_debug_overlay(
        debug_systems.debug_ui.get_overlay(sid_fps),
        &game_state.fps_debug,
        (1000.
            / game_state
                .cvars
                .gameplay_update_tick_ms
                .read(&engine_state.config)) as u64,
        game_state.cvars.vsync.read(&engine_state.config),
    );

    let draw_mouse_rulers = game_state
        .debug_cvars
        .draw_mouse_rulers
        .read(&engine_state.config);
    if draw_mouse_rulers {
        let painter = &mut debug_systems.global_painter;
        update_mouse_debug_overlay(
            debug_systems.debug_ui.get_overlay(sid_mouse),
            painter,
            &game_state.window,
        );
    }

    update_win_debug_overlay(
        debug_systems.debug_ui.get_overlay(sid_window),
        &game_state.window,
    );

    let draw_fps_graph = game_state
        .debug_cvars
        .draw_fps_graph
        .read(&engine_state.config);
    debug_systems
        .debug_ui
        .set_graph_enabled(sid_fps, draw_fps_graph);
    if draw_fps_graph {
        update_graph_fps(
            debug_systems.debug_ui.get_graph(sid_fps),
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
        .set_graph_enabled(sid_prev_frame_time, draw_prev_frame_t_graph);
    if draw_prev_frame_t_graph {
        update_graph_prev_frame_t(
            debug_systems.debug_ui.get_graph(sid_prev_frame_time),
            &engine_state.time,
            &engine_state.prev_frame_time,
        );
    }

    ////// Per-Level debugs //////
    let painters = &mut debug_systems.painters;
    let debug_ui = &mut debug_systems.debug_ui;
    let target_win_size = engine_state.app_config.target_win_size;

    let cvars = &game_state.debug_cvars;
    let draw_entities = cvars.draw_entities.read(&engine_state.config);
    let draw_component_lists = cvars.draw_component_lists.read(&engine_state.config);
    let draw_velocities = cvars.draw_velocities.read(&engine_state.config);
    let draw_entity_prev_frame_ghost = cvars
        .draw_entity_prev_frame_ghost
        .read(&engine_state.config);
    let draw_colliders = cvars.draw_colliders.read(&engine_state.config);
    let draw_debug_grid = cvars.draw_debug_grid.read(&engine_state.config);
    let draw_comp_alloc_colliders = cvars.draw_comp_alloc_colliders.read(&engine_state.config);
    let square_size = cvars.debug_grid_square_size.read(&engine_state.config);
    let opacity = cvars.debug_grid_opacity.read(&engine_state.config) as u8;
    let draw_world_chunks = cvars.draw_world_chunks.read(&engine_state.config);
    let draw_lights = cvars.draw_lights.read(&engine_state.config);
    let lv_batches = &mut game_state.level_batches;
    let global_painter = &mut debug_systems.global_painter;

    game_state
        .gameplay_system
        .levels
        .foreach_active_level(|level| {
            let debug_painter = painters
                .get_mut(&level.id)
                .unwrap_or_else(|| fatal!("Debug painter not found for level {:?}", level.id));

            update_entities_debug_overlay(debug_ui.get_overlay(sid_entities), &level.world);

            update_camera_debug_overlay(debug_ui.get_overlay(sid_camera), &level.get_camera());

            update_physics_debug_overlay(
                debug_ui.get_overlay(sid_physics),
                &collision_debug_data[&level.id],
                &level.chunks,
            );

            if draw_entities {
                debug_draw_transforms(debug_painter, &level.world);
            }

            if draw_velocities {
                debug_draw_velocities(debug_painter, &level.world);
            }

            if draw_entity_prev_frame_ghost {
                let batches = lv_batches.get_mut(&level.id).unwrap();
                debug_draw_entities_prev_frame_ghost(batches, &mut level.world);
            }

            if draw_component_lists {
                debug_draw_component_lists(debug_painter, &level.world);
            }

            if draw_colliders {
                debug_draw_colliders(debug_painter, &level.world);
            }

            if draw_lights {
                debug_draw_lights(global_painter, debug_painter, &level.lights);
            }

            // Debug grid
            if draw_debug_grid {
                debug_draw_grid(
                    debug_painter,
                    &level.get_camera().transform,
                    target_win_size,
                    square_size,
                    opacity,
                );
            }

            if draw_world_chunks {
                level.chunks.debug_draw(debug_painter);
            }

            if draw_comp_alloc_colliders {
                use ecs_engine::collisions::collider::Collider;
                ecs_engine::ecs::ecs_world::draw_comp_alloc::<Collider>(
                    &level.world,
                    global_painter,
                );
            }
        });
}

#[cfg(debug_assertions)]
fn update_joystick_debug_overlay(
    debug_overlay: &mut debug::overlay::Debug_Overlay,
    joy_state: &input::joystick_state::Joystick_State,
    input_cfg: crate::input_utils::Input_Config,
    cfg: &ecs_engine::cfg::Config,
) {
    use input::bindings::joystick;

    debug_overlay.clear();

    let deadzone = input_cfg.joy_deadzone.read(cfg);

    let (real_axes, joy_mask) = input::joystick_state::all_joysticks_values(joy_state);

    for (joy_id, axes) in real_axes.iter().enumerate() {
        if (joy_mask & (1 << joy_id)) != 0 {
            debug_overlay.add_line_color(&format!("> Joy {} <", joy_id), colors::rgb(235, 52, 216));

            for i in 0u8..joystick::Joystick_Axis::_Count as u8 {
                let axis: joystick::Joystick_Axis = i.try_into().unwrap_or_else(|err| {
                    fatal!("Failed to convert {} to a valid Joystick_Axis: {}", i, err)
                });
                debug_overlay.add_line_color(
                    &format!("{:?}: {:5.2}", axis, axes[i as usize]),
                    if axes[i as usize].abs() > deadzone {
                        colors::GREEN
                    } else {
                        colors::YELLOW
                    },
                );
            }
        }
    }
}

#[cfg(debug_assertions)]
fn update_time_debug_overlay(debug_overlay: &mut debug::overlay::Debug_Overlay, time: &time::Time) {
    debug_overlay.clear();

    debug_overlay.add_line_color(
        &format!(
            "[time] game: {:.2}, real: {:.2}, scale: {:.2}, paused: {}",
            time.get_game_time().as_secs_f32(),
            time.get_real_time().as_secs_f32(),
            time.time_scale,
            if time.paused { "yes" } else { "no" }
        ),
        colors::rgb(100, 200, 200),
    );
}

#[cfg(debug_assertions)]
fn update_fps_debug_overlay(
    debug_overlay: &mut debug::overlay::Debug_Overlay,
    fps: &debug::fps::Fps_Counter,
    target_fps: u64,
    vsync: bool,
) {
    debug_overlay.clear();
    debug_overlay.add_line_color(
        &format!(
            "FPS: {} (target ~{}, vsync {})",
            fps.get_fps() as u32,
            target_fps,
            if vsync { "on" } else { "off" },
        ),
        colors::rgba(180, 180, 180, 200),
    );
}

#[cfg(debug_assertions)]
fn update_mouse_debug_overlay(
    debug_overlay: &mut debug::overlay::Debug_Overlay,
    painter: &mut Debug_Painter,
    window: &Render_Window_Handle,
) {
    use ecs_engine::common::shapes::Line;

    debug_overlay.clear();
    let pos = window::mouse_pos_in_window(window);
    debug_overlay.position = Vec2f::from(pos) + v2!(0., -15.);
    debug_overlay.add_line_color(
        &format!("{},{}", pos.x, pos.y),
        colors::rgba(220, 220, 220, 220),
    );

    let color = colors::rgba(255, 255, 255, 150);
    let (win_w, win_h) = window::get_window_real_size(window);
    let from_x = Vec2f::new(0., pos.y as _);
    let to_x = Vec2f::new(win_w as _, pos.y as _);
    let from_y = Vec2f::new(pos.x as _, 0.);
    let to_y = Vec2f::new(pos.x as _, win_h as _);
    painter.add_line(
        Line {
            from: from_x,
            to: to_x,
            thickness: 1.0,
        },
        color,
    );
    painter.add_line(
        Line {
            from: from_y,
            to: to_y,
            thickness: 1.0,
        },
        color,
    );
}

#[cfg(debug_assertions)]
fn update_win_debug_overlay(
    debug_overlay: &mut debug::overlay::Debug_Overlay,
    window: &Render_Window_Handle,
) {
    let tsize = window::get_window_target_size(window);
    let rsize = window::get_window_real_size(window);
    debug_overlay.clear();
    debug_overlay.add_line_color(
        &format!(
            "WinSize: target = {}x{}, real = {}x{}",
            tsize.0, tsize.1, rsize.0, rsize.1
        ),
        colors::rgba(110, 190, 250, 220),
    );
}

#[cfg(debug_assertions)]
fn update_entities_debug_overlay(
    debug_overlay: &mut debug::overlay::Debug_Overlay,
    ecs_world: &Ecs_World,
) {
    debug_overlay.clear();
    debug_overlay.add_line_color(
        &format!("Entities: {}", ecs_world.entities().len()),
        colors::rgba(220, 100, 180, 220),
    );
}

#[cfg(debug_assertions)]
fn update_camera_debug_overlay(
    debug_overlay: &mut debug::overlay::Debug_Overlay,
    camera: &ecs_engine::ecs::components::gfx::C_Camera2D,
) {
    debug_overlay.clear();
    debug_overlay.add_line_color(
        &format!(
            "[cam] pos: {:.2},{:.2}, scale: {:.1}",
            camera.transform.position().x,
            camera.transform.position().y,
            camera.transform.scale().x
        ),
        colors::rgba(220, 180, 100, 220),
    );
}

#[cfg(debug_assertions)]
fn update_physics_debug_overlay(
    debug_overlay: &mut debug::overlay::Debug_Overlay,
    collision_data: &physics::Collision_System_Debug_Data,
    chunks: &crate::spatial::World_Chunks,
) {
    debug_overlay.clear();
    debug_overlay.add_line_color(
        &format!(
            "[phys] n_inter_tests: {}, n_chunks: {}",
            collision_data.n_intersection_tests,
            chunks.n_chunks(),
        ),
        colors::rgba(0, 173, 90, 220),
    );
}

#[cfg(debug_assertions)]
fn update_record_debug_overlay(
    debug_overlay: &mut debug::overlay::Debug_Overlay,
    recording: bool,
    replaying: bool,
) {
    debug_overlay.clear();
    if replaying {
        debug_overlay.add_line_color("REPLAYING", colors::rgb(30, 200, 30));
    } else if recording {
        debug_overlay.add_line_color("RECORDING", colors::rgb(200, 30, 30));
    }
}

#[cfg(debug_assertions)]
fn debug_draw_colliders(debug_painter: &mut Debug_Painter, ecs_world: &Ecs_World) {
    use crate::collisions::Game_Collision_Layer;
    use ecs_engine::collisions::collider::{Collider, Collision_Shape};
    use std::convert::TryFrom;

    foreach_entity!(ecs_world, +Collider, +C_Spatial2D, |entity| {
        let collider = ecs_world.get_component::<Collider>(entity).unwrap();
        // Note: since our collision detector doesn't handle rotation, draw the colliders with rot = 0
        // @Incomplete: scale?
        let mut transform = Transform2D::from_pos_rot_scale(collider.position + collider.offset, rad(0.), v2!(1., 1.));

        let color = if collider.colliding_with.is_some() {
            colors::rgba(255, 0, 0, 100)
        } else {
            colors::rgba(255, 255, 0, 100)
        };

        match collider.shape {
            Collision_Shape::Rect { width, height } => {
                transform.translate(-width * 0.5, -height * 0.5);
                debug_painter.add_rect(Vec2f::new(width, height), &transform, color);
            }
            Collision_Shape::Circle { radius } => {
                transform.translate(-radius * 0.5, -radius * 0.5);
                debug_painter.add_circle(
                    Circle {
                        center: transform.position(),
                        radius,
                    },
                    color,
                );
            }
            _ => {}
        }

        debug_painter.add_text(
            &Game_Collision_Layer::try_from(collider.layer).map_or_else(
                |_| format!("? {}", collider.layer),
                |gcl| format!("{:?}", gcl),
            ),
            transform.position(),
            8,
            colors::BLACK);
    });
}

#[cfg(debug_assertions)]
fn debug_draw_transforms(debug_painter: &mut Debug_Painter, ecs_world: &Ecs_World) {
    foreach_entity!(ecs_world, +C_Spatial2D, |entity| {
        let spatial = ecs_world.get_component::<C_Spatial2D>(entity).unwrap();
        let transform = &spatial.transform;
        debug_painter.add_circle(
            Circle {
                radius: 5.,
                center: transform.position(),
            },
            colors::rgb(50, 100, 200),
        );

        debug_painter.add_text(
            &format!(
                "{:.2},{:.2}",
                transform.position().x,
                transform.position().y
            ),
            transform.position(),
            10,
            Paint_Properties {
                color: colors::WHITE,
                border_thick: 1.,
                border_color: colors::BLACK,
                ..Default::default()
            },
        );
    });
}

#[cfg(debug_assertions)]
fn debug_draw_velocities(debug_painter: &mut Debug_Painter, ecs_world: &Ecs_World) {
    const COLOR: colors::Color = colors::rgb(100, 0, 120);

    foreach_entity!(ecs_world, +C_Spatial2D, |entity| {
        let spatial = ecs_world.get_component::<C_Spatial2D>(entity).unwrap();
        let transform = &spatial.transform;
        debug_painter.add_arrow(
            Arrow {
                center: transform.position(),
                direction: spatial.velocity,
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
    });
}

#[cfg(debug_assertions)]
fn debug_draw_component_lists(debug_painter: &mut Debug_Painter, ecs_world: &Ecs_World) {
    use crate::debug::entity_debug::C_Debug_Data;
    use ecs_engine::common::bitset::Bit_Set;
    use std::borrow::Borrow;

    foreach_entity!(ecs_world, |entity| {
        let pos = if let Some(spatial) = ecs_world.get_component::<C_Spatial2D>(entity) {
            spatial.transform.position() + v2!(0., -15.)
        } else {
            v2!(0., 0.) // @Incomplete
        };

        let name = if let Some(debug) = ecs_world.get_component::<C_Debug_Data>(entity) {
            debug.entity_name
        } else {
            "Unknown"
        };
        debug_painter.add_shaded_text(name, pos, 8, colors::GREEN, colors::BLACK);

        let comp_set = ecs_world.get_entity_comp_set(entity);
        let comp_set_b: &Bit_Set = comp_set.borrow();
        for (i, handle) in comp_set_b.into_iter().enumerate() {
            debug_painter.add_shaded_text(
                &format!(
                    " {}",
                    ecs_world::component_name_from_handle(
                        ecs_world,
                        handle as ecs_world::Component_Handle
                    )
                    .unwrap()
                ),
                pos + v2!(0., (i + 1) as f32 * 8.5),
                8,
                colors::WHITE,
                colors::BLACK,
            );
        }
    });
}

#[cfg(debug_assertions)]
fn debug_draw_lights(
    screenspace_debug_painter: &mut Debug_Painter,
    debug_painter: &mut Debug_Painter,
    lights: &ecs_engine::gfx::light::Lights,
) {
    screenspace_debug_painter.add_shaded_text(
        &format!(
            "Ambient Light: color: #{:X}, intensity: {}",
            colors::color_to_hex_no_alpha(lights.ambient_light.color),
            lights.ambient_light.intensity
        ),
        v2!(5., 300.),
        15,
        lights.ambient_light.color,
        if colors::to_hsv(lights.ambient_light.color).v > 0.5 {
            colors::BLACK
        } else {
            colors::WHITE
        },
    );
    for pl in &lights.point_lights {
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
}

#[cfg(debug_assertions)]
fn debug_draw_entities_prev_frame_ghost(
    batches: &mut gfx::render::batcher::Batches,
    ecs_world: &mut Ecs_World,
) {
    use crate::debug::entity_debug::C_Debug_Data;
    use ecs_engine::ecs::components::gfx::C_Renderable;
    use ecs_engine::gfx::render;

    foreach_entity!(ecs_world, +C_Spatial2D, +C_Renderable, +C_Debug_Data, |entity| {
        let frame_starting_pos = ecs_world.get_component::<C_Spatial2D>(entity).unwrap().frame_starting_pos;
        let C_Renderable {
            material,
            rect,
            modulate,
            z_index,
        } = *ecs_world.get_component::<C_Renderable>(entity).unwrap();

        let debug_data = ecs_world.get_component_mut::<C_Debug_Data>(entity).unwrap();
        if (debug_data.n_prev_positions_filled as usize) < debug_data.prev_positions.len() {
            debug_data.prev_positions[debug_data.n_prev_positions_filled as usize] = frame_starting_pos;
            debug_data.n_prev_positions_filled += 1;
        } else {
            for i in 0..debug_data.prev_positions.len() - 1 {
                debug_data.prev_positions[i] = debug_data.prev_positions[i + 1];
            }
            debug_data.prev_positions[debug_data.prev_positions.len() - 1] = frame_starting_pos;
        }

        for i in 0..debug_data.n_prev_positions_filled {
            let transform = Transform2D::from_pos(debug_data.prev_positions[i as usize]);
            let color = colors::rgba(
                modulate.r,
                modulate.g,
                modulate.b,
                200 - 10 * (debug_data.prev_positions.len() - i as usize) as u8,
            );
            render::render_texture_ws(batches, material, &rect, color, &transform, z_index);
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
) {
    let Vec2f { x: cx, y: cy } = camera_transform.position();
    let Vec2f {
        x: cam_sx,
        y: cam_sy,
    } = camera_transform.scale();
    let n_horiz = (screen_width as f32 * cam_sx / square_size).floor() as usize + 2;
    let n_vert = (screen_height as f32 * cam_sy / square_size).floor() as usize + 2;
    let col_gray = colors::rgba(200, 200, 200, grid_opacity);
    let col_white = colors::rgba(255, 255, 255, grid_opacity);
    let sq_coord = Vec2f::new(
        (cx / square_size).floor() * square_size,
        (cy / square_size).floor() * square_size,
    );

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
            debug_painter.add_text(
                &format!("{},{}", pos.x, pos.y),
                pos + Vec2f::new(5., 5.),
                (square_size as i32 / 6).max(8) as u16,
                color,
            );
        }
    }
}

#[cfg(debug_assertions)]
fn update_graph_fps(
    graph: &mut debug::graph::Debug_Graph_View,
    time: &time::Time,
    fps: &debug::fps::Fps_Counter,
) {
    const TIME_LIMIT: f32 = 60.0;

    let fps = fps.get_instant_fps();
    debug::graph::add_point_and_scroll(graph, time.get_real_time(), TIME_LIMIT, fps);
}

#[cfg(debug_assertions)]
fn update_graph_prev_frame_t(
    graph: &mut debug::graph::Debug_Graph_View,
    time: &time::Time,
    prev_frame_t: &Duration,
) {
    const TIME_LIMIT: f32 = 10.0;

    debug::graph::add_point_and_scroll(
        graph,
        time.get_real_time(),
        TIME_LIMIT,
        prev_frame_t.as_secs_f32() * 1000.,
    );
}
