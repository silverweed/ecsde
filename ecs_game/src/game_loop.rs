use super::{Game_Resources, Game_State};
use ecs_engine::common::colors;
use ecs_engine::common::transform::Transform2D;
use ecs_engine::common::Maybe_Error;
use ecs_engine::core::app;
use ecs_engine::core::sleep;
use ecs_engine::core::time;
use ecs_engine::gfx;
use ecs_engine::gfx::render_system;
use ecs_engine::input;
use ecs_engine::resources::gfx::Gfx_Resources;
use std::convert::TryInto;
use std::time::{Duration, Instant};

#[cfg(debug_assertions)]
use ecs_engine::{
    common::angle::rad, common::stringid::String_Id, common::vector::Vec2f, debug,
    debug::painter::Debug_Painter, gfx::paint_props::Paint_Properties, gfx::window,
};

pub fn tick_game<'a>(
    game_state: &'a mut Game_State<'a>,
    game_resources: &'a mut Game_Resources<'a>,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!("tick_game");

    game_state.engine_state.cur_frame += 1;
    game_state.engine_state.time.update();
    let real_dt = game_state.engine_state.time.real_dt();

    // @Speed @WaitForStable: these should all be computed at compile time.
    // Probably will do that when either const fn or proc macros/syntax extensions are stable.
    #[cfg(debug_assertions)]
    let (sid_joysticks, sid_msg) = (String_Id::from("joysticks"), String_Id::from("msg"));

    #[cfg(debug_assertions)]
    let debug_systems = &mut game_state.engine_state.debug_systems;

    let target_time_per_frame = Duration::from_micros(
        (game_state
            .cvars
            .gameplay_update_tick_ms
            .read(&game_state.engine_state.config)
            * 1000.0) as u64,
    );
    let t_before_work = Instant::now();

    // Check if the replay ended this frame
    if game_state.is_replaying && game_state.input_provider.is_realtime_player_input() {
        #[cfg(debug_assertions)]
        debug_systems
            .debug_ui
            .get_fadeout_overlay(sid_msg)
            .add_line("REPLAY HAS ENDED.");
        game_state.is_replaying = false;
    }

    // Update input
    {
        trace!("input_system::update");

        let process_game_actions;
        #[cfg(debug_assertions)]
        {
            process_game_actions =
                debug_systems.console.status != debug::console::Console_Status::Open;
        }
        #[cfg(not(debug_assertions))]
        {
            process_game_actions = true;
        }

        input::input_system::update_input(
            &mut game_state.engine_state.input_state,
            &mut game_state.window,
            &mut *game_state.input_provider,
            &game_state.engine_state.config,
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
                .update(&game_state.engine_state.input_state.raw_events);

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

        scroller.handle_events(&game_state.engine_state.input_state.raw_events);

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
                .core_actions
                .split_off(0),
            &mut game_state.window,
        ) {
            game_state.engine_state.should_close = true;
            return Ok(());
        }
    }

    {
        let actions = game_state
            .engine_state
            .input_state
            .game_actions
            .split_off(0);

        #[cfg(debug_assertions)]
        let input_state = &game_state.engine_state.input_state;
        #[cfg(debug_assertions)]
        let raw_events = &input_state.raw_events;
        #[cfg(debug_assertions)]
        let (real_axes, joy_mask) =
            input::joystick_state::all_joysticks_values(&input_state.joy_state);

        #[cfg(debug_assertions)]
        {
            update_joystick_debug_overlay(
                debug_systems.debug_ui.get_overlay(sid_joysticks),
                real_axes,
                joy_mask,
                game_state.gameplay_system.input_cfg,
                &game_state.engine_state.config,
            );

            // Record replay data (only if we're not already playing back a replay).
            if debug_systems.replay_recording_system.is_recording()
                && game_state.input_provider.is_realtime_player_input()
            {
                let record_replay_data = game_state
                    .debug_cvars
                    .record_replay
                    .read(&game_state.engine_state.config);
                if record_replay_data {
                    debug_systems
                        .replay_recording_system
                        .update(raw_events, real_axes, joy_mask);
                }
            }
        }

        // Update states
        {
            trace!("state_mgr::handle_actions");
            if game_state.state_mgr.handle_actions(
                &actions,
                &mut game_state.engine_state,
                &mut game_state.gameplay_system,
            ) {
                game_state.engine_state.should_close = true;
                return Ok(());
            }
        }

        #[cfg(debug_assertions)]
        {
            if actions.contains(&(
                String_Id::from("toggle_console"),
                ecs_engine::input::input_system::Action_Kind::Pressed,
            )) {
                game_state.engine_state.debug_systems.console.toggle();
            }
        }

        // Update game systems
        {
            trace!("game_update");

            let axes = &game_state.engine_state.input_state.axes;

            game_state.gameplay_system.realtime_update(
                &real_dt,
                &game_state.engine_state.time,
                &actions,
                axes,
                &game_state.engine_state.config,
            );

            let time = &game_state.engine_state.time;
            let update_dt = time::mul_duration(
                &target_time_per_frame,
                time.time_scale * (!time.paused as u32 as f32),
            );
            game_state.gameplay_system.update(
                &update_dt,
                &game_state.engine_state.time,
                &actions,
                axes,
                &game_state.engine_state.config,
            );
        }
    }

    // Update collisions
    {
        trace!("collision_system::update");

        let gameplay_system = &mut game_state.gameplay_system;
        let collision_system = &mut game_state.engine_state.systems.collision_system;
        gameplay_system.foreach_active_level(|level| {
            collision_system.update(&mut level.world);
        });
    }

    // Update audio
    {
        trace!("audio_system_update");
        game_state.engine_state.systems.audio_system.update();
    }

    #[cfg(debug_assertions)]
    update_debug(game_state);

    update_graphics(game_state, &mut game_resources.gfx, real_dt)?;

    #[cfg(debug_assertions)]
    {
        game_state.engine_state.config.update();
        game_state.fps_debug.tick(&real_dt);
    }

    {
        trace!("wait_end_frame");

        let mut t_elapsed_for_work = t_before_work.elapsed();
        if t_elapsed_for_work < target_time_per_frame {
            while t_elapsed_for_work < target_time_per_frame {
                if let Some(granularity) = game_state.sleep_granularity {
                    if granularity < target_time_per_frame - t_elapsed_for_work {
                        let gra_ns = granularity.as_nanos();
                        let rem_ns = (target_time_per_frame - t_elapsed_for_work).as_nanos();
                        let time_to_sleep =
                            Duration::from_nanos((rem_ns / gra_ns).try_into().unwrap());
                        sleep::sleep(time_to_sleep);
                    }
                }

                t_elapsed_for_work = t_before_work.elapsed();
            }
        } else {
            lerr!(
                "Frame budget exceeded! At frame {}: {} / {} ms",
                game_state.engine_state.cur_frame,
                time::to_ms_frac(&t_elapsed_for_work),
                time::to_ms_frac(&target_time_per_frame)
            );
        }
    }

    {
        trace!("display");
        gfx::window::display(&mut game_state.window);
    }

    Ok(())
}

fn update_graphics(
    game_state: &mut Game_State,
    gres: &mut Gfx_Resources,
    real_dt: Duration,
) -> Maybe_Error {
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

        gfx::window::set_clear_color(window, colors::rgb(0, 0, 0));
        gfx::window::clear(window);
    }

    {
        trace!("clear_batches");
        gfx::render::batcher::clear_batches(&mut game_state.engine_state.global_batches);
        for batches in game_state.level_batches.values_mut() {
            gfx::render::batcher::clear_batches(batches);
        }
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
        gameplay_system.foreach_active_level(|level| {
            let render_args = render_system::Render_System_Update_Args {
                batches: batches.get_mut(&level.id).unwrap(),
                ecs_world: &level.world,
                frame_alloc,
                cfg: render_cfg,
            };

            render_system::update(render_args);
        });
    }

    #[cfg(debug_assertions)]
    {
        // Draw debug painter (one per active level)
        let painters = &mut game_state.engine_state.debug_systems.painters;
        let window = &mut game_state.window;
        let batches = &mut game_state.level_batches;
        game_state.gameplay_system.foreach_active_level(|level| {
            let painter = painters
                .get_mut(&level.id)
                .unwrap_or_else(|| panic!("Debug painter not found for level {:?}", level.id));
            let batches = batches.get_mut(&level.id).unwrap();
            painter.draw(window, gres, batches, &level.get_camera().transform);
            painter.clear();
        });

        // Global painter
        let painter = game_state.engine_state.debug_systems.global_painter();
        painter.draw(
            &mut game_state.window,
            gres,
            &mut game_state.engine_state.global_batches,
            &Transform2D::default(),
        );
        painter.clear();

        // Draw debug UI
        {
            trace!("debug_ui::update");
            game_state
                .engine_state
                .debug_systems
                .debug_ui
                .update_and_draw(
                    &real_dt,
                    &mut game_state.window,
                    gres,
                    &mut game_state.engine_state.global_batches,
                    &game_state.engine_state.debug_systems.log,
                    &mut game_state.engine_state.frame_alloc,
                );
        }

        // Draw console
        {
            trace!("console::draw");
            game_state.engine_state.debug_systems.console.draw(
                &mut game_state.window,
                gres,
                &mut game_state.engine_state.global_batches,
            );
        }
    }

    let lv_batches = &mut game_state.level_batches;
    let window = &mut game_state.window;
    game_state.gameplay_system.foreach_active_level(|level| {
        gfx::render::batcher::draw_batches(
            window,
            &gres,
            lv_batches.get_mut(&level.id).unwrap(),
            &level.get_camera().transform,
        );
    });
    gfx::render::batcher::draw_batches(
        window,
        &gres,
        &mut game_state.engine_state.global_batches,
        &Transform2D::default(),
    );

    Ok(())
}

#[cfg(debug_assertions)]
fn update_debug(game_state: &mut Game_State) {
    let engine_state = &mut game_state.engine_state;
    let debug_systems = &mut engine_state.debug_systems;

    // @Speed @WaitForStable: these should all be computed at compile time.
    let (sid_time, sid_fps, sid_entities, sid_camera, sid_mouse, sid_window) = (
        String_Id::from("time"),
        String_Id::from("fps"),
        String_Id::from("entities"),
        String_Id::from("camera"),
        String_Id::from("mouse"),
        String_Id::from("window"),
    );

    // Frame scroller

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
        let painter = debug_systems
            .painters
            .get_mut(&String_Id::from(""))
            .unwrap();
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
        debug_systems.debug_ui.set_graph_enabled(sid_fps, true);
        update_graph_fps(
            debug_systems.debug_ui.get_graph(sid_fps),
            &engine_state.time,
            &game_state.fps_debug,
        );
    }

    ////// Per-Level debugs //////
    let painters = &mut debug_systems.painters;
    let collision_system = &engine_state.systems.collision_system;
    let ui_system = &mut debug_systems.debug_ui;
    let target_win_size = engine_state.app_config.target_win_size;

    let cvars = &game_state.debug_cvars;
    let draw_entities = cvars.draw_entities.read(&engine_state.config);
    let draw_velocities = cvars.draw_velocities.read(&engine_state.config);
    let draw_colliders = cvars.draw_colliders.read(&engine_state.config);
    let draw_collision_quadtree = cvars.draw_collision_quadtree.read(&engine_state.config);
    let draw_debug_grid = cvars.draw_debug_grid.read(&engine_state.config);
    let square_size = cvars.debug_grid_square_size.read(&engine_state.config);
    let opacity = cvars.debug_grid_opacity.read(&engine_state.config) as u8;

    game_state.gameplay_system.foreach_active_level(|level| {
        let debug_painter = painters
            .get_mut(&level.id)
            .unwrap_or_else(|| fatal!("Debug painter not found for level {:?}", level.id));

        update_entities_debug_overlay(ui_system.get_overlay(sid_entities), &level.world);

        update_camera_debug_overlay(ui_system.get_overlay(sid_camera), &level.get_camera());

        if draw_entities {
            debug_draw_transforms(debug_painter, &level.world);
        }

        if draw_velocities {
            debug_draw_velocities(debug_painter, &level.world);
        }

        if draw_colliders {
            debug_draw_colliders(debug_painter, &level.world);
        }

        if draw_collision_quadtree {
            collision_system.debug_draw_quadtree(debug_painter);

            collision_system.debug_draw_entities_quad_id(&level.world, debug_painter);
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

        collision_system.debug_draw_applied_impulses(debug_painter);
    });
}

#[cfg(debug_assertions)]
fn update_joystick_debug_overlay(
    debug_overlay: &mut debug::overlay::Debug_Overlay,
    real_axes: &[input::joystick_state::Real_Axes_Values;
         input::bindings::joystick::JOY_COUNT as usize],
    joy_mask: u8,
    input_cfg: crate::input_utils::Input_Config,
    cfg: &ecs_engine::cfg::Config,
) {
    use input::bindings::joystick;

    debug_overlay.clear();

    let deadzone = input_cfg.joy_deadzone.read(cfg);

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
    fps: &debug::fps::Fps_Console_Printer,
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
    window: &window::Window_Handle,
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
    window: &window::Window_Handle,
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
    ecs_world: &ecs_engine::ecs::ecs_world::Ecs_World,
) {
    debug_overlay.clear();
    debug_overlay.add_line_color(
        &format!("Entities: {}", ecs_world.entity_manager.n_live_entities()),
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
fn debug_draw_colliders(
    debug_painter: &mut Debug_Painter,
    ecs_world: &ecs_engine::ecs::ecs_world::Ecs_World,
) {
    use ecs_engine::collisions::collider::{Collider, Collider_Shape};
    use ecs_engine::common::shapes;
    use ecs_engine::ecs::components::base::C_Spatial2D;

    let mut stream = ecs_engine::ecs::entity_stream::new_entity_stream(ecs_world)
        .require::<Collider>()
        .require::<C_Spatial2D>()
        .build();
    loop {
        let entity = stream.next(ecs_world);
        if entity.is_none() {
            break;
        }
        let entity = entity.unwrap();
        let collider = ecs_world.get_component::<Collider>(entity).unwrap();
        let transform = &ecs_world
            .get_component::<C_Spatial2D>(entity)
            .unwrap()
            .global_transform;
        // Note: since our collision detector doesn't handle rotation, draw the colliders with rot = 0
        let mut transform = *transform;
        transform.set_rotation(rad(0.));
        transform.translate_v(collider.offset);

        let color = if collider.colliding {
            colors::rgba(255, 0, 0, 100)
        } else {
            colors::rgba(255, 255, 0, 100)
        };

        match collider.shape {
            Collider_Shape::Rect { width, height } => {
                debug_painter.add_rect(Vec2f::new(width, height), &transform, color);
            }
            Collider_Shape::Circle { radius } => {
                debug_painter.add_circle(
                    shapes::Circle {
                        center: transform.position(),
                        radius,
                    },
                    color,
                );
            }
        }
    }
}

#[cfg(debug_assertions)]
fn debug_draw_transforms(
    debug_painter: &mut Debug_Painter,
    ecs_world: &ecs_engine::ecs::ecs_world::Ecs_World,
) {
    use ecs_engine::common::shapes::Circle;
    use ecs_engine::ecs::components::base::C_Spatial2D;

    let mut stream = ecs_engine::ecs::entity_stream::new_entity_stream(ecs_world)
        .require::<C_Spatial2D>()
        .build();
    loop {
        let entity = stream.next(ecs_world);
        if entity.is_none() {
            break;
        }
        let entity = entity.unwrap();
        let spatial = ecs_world.get_component::<C_Spatial2D>(entity).unwrap();
        let transform = &spatial.global_transform;
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
    }
}

#[cfg(debug_assertions)]
fn debug_draw_velocities(
    debug_painter: &mut Debug_Painter,
    ecs_world: &ecs_engine::ecs::ecs_world::Ecs_World,
) {
    use ecs_engine::common::shapes::Arrow;
    use ecs_engine::ecs::components::base::C_Spatial2D;

    const COLOR: colors::Color = colors::rgb(100, 0, 120);

    let mut stream = ecs_engine::ecs::entity_stream::new_entity_stream(ecs_world)
        .require::<C_Spatial2D>()
        .build();
    loop {
        let entity = stream.next(ecs_world);
        if entity.is_none() {
            break;
        }
        let entity = entity.unwrap();
        let spatial = ecs_world.get_component::<C_Spatial2D>(entity).unwrap();
        let transform = &spatial.global_transform;
        debug_painter.add_arrow(
            Arrow {
                center: transform.position(),
                direction: spatial.velocity,
                thickness: 3.,
                arrow_size: 20.,
            },
            COLOR,
        );
        debug_painter.add_text(
            &spatial.velocity.to_string(),
            transform.position() + Vec2f::new(1., -14.),
            12,
            colors::WHITE,
        );
        debug_painter.add_text(
            &spatial.velocity.to_string(),
            transform.position() + Vec2f::new(0., -15.),
            12,
            COLOR,
        );
    }
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
        ((cx * cam_sx) / square_size).floor() * square_size,
        ((cy * cam_sy) / square_size).floor() * square_size,
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
    fps: &debug::fps::Fps_Console_Printer,
) {
    const TIME_LIMIT: f32 = 60.0;

    let fps = fps.get_fps();
    let now = time.get_real_time().as_secs_f32();
    graph.data.x_range.end = now;
    if graph.data.x_range.end - graph.data.x_range.start > TIME_LIMIT {
        graph.data.x_range.start = graph.data.x_range.end - TIME_LIMIT;
    }
    graph.data.add_point(now, fps);
}
