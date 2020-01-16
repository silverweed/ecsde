use super::{Game_Resources, Game_State};
use ecs_engine::core::app;
use ecs_engine::core::common::colors;
use ecs_engine::core::common::vector::Vec2f;
use ecs_engine::core::common::Maybe_Error;
use ecs_engine::core::time;
use ecs_engine::gfx;
use ecs_engine::gfx::render_system;
use ecs_engine::resources::gfx::Gfx_Resources;
use std::time::Duration;

#[cfg(debug_assertions)]
use ecs_engine::core::common::stringid::String_Id;
#[cfg(debug_assertions)]
use ecs_engine::core::common::transform::Transform2D;
#[cfg(debug_assertions)]
use ecs_engine::debug;
#[cfg(debug_assertions)]
use ecs_engine::debug::debug_painter::Debug_Painter;
#[cfg(debug_assertions)]
use ecs_engine::gfx::render::Paint_Properties;
#[cfg(debug_assertions)]
use ecs_engine::input;

pub fn tick_game<'a>(
    game_state: &'a mut Game_State<'a>,
    game_resources: &'a mut Game_Resources<'a>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let tracer = clone_tracer!(game_state.engine_state.tracer);
    trace!("tick_game", tracer);

    // @Speed: these should all be computed at compile time.
    // Probably will do that when either const fn or proc macros/syntax extensions are stable.
    #[cfg(debug_assertions)]
    let (sid_joysticks, sid_msg, sid_time, sid_fps, sid_entities, sid_camera) = (
        String_Id::from("joysticks"),
        String_Id::from("msg"),
        String_Id::from("time"),
        String_Id::from("fps"),
        String_Id::from("entities"),
        String_Id::from("camera"),
    );

    let window = &mut game_state.window;
    let engine_state = &mut game_state.engine_state;
    engine_state.time.update();

    let dt = engine_state.time.dt();
    let real_dt = engine_state.time.real_dt();
    let systems = &mut engine_state.systems;
    #[cfg(debug_assertions)]
    let debug_systems = &mut engine_state.debug_systems;

    #[cfg(debug_assertions)]
    update_time_debug_overlay(
        debug_systems.debug_ui_system.get_overlay(sid_time),
        &engine_state.time,
    );
    let update_time = Duration::from_millis(
        game_state
            .gameplay_update_tick_ms
            .read(&engine_state.config) as u64,
    );

    game_state.execution_time += dt;

    // Check if the replay ended this frame
    if game_state.is_replaying && game_state.input_provider.is_realtime_player_input() {
        #[cfg(debug_assertions)]
        debug_systems
            .debug_ui_system
            .get_fadeout_overlay(sid_msg)
            .add_line("REPLAY HAS ENDED.");
        game_state.is_replaying = false;
    }

    // Update input
    {
        trace!("input_system::update", tracer);
        systems.input_system.update(
            window,
            &mut *game_state.input_provider,
            &engine_state.config,
        );
    }

    // Handle actions
    {
        trace!("app::handle_core_actions", tracer);
        if app::handle_core_actions(&systems.input_system.extract_core_actions(), window) {
            engine_state.should_close = true;
            return Ok(false);
        }
    }

    {
        let actions = systems.input_system.extract_game_actions();

        #[cfg(debug_assertions)]
        let input_system = &systems.input_system;
        #[cfg(debug_assertions)]
        let raw_events = input_system.get_raw_events();
        #[cfg(debug_assertions)]
        let (real_axes, joy_mask) = input_system.get_all_real_axes();

        #[cfg(debug_assertions)]
        {
            update_joystick_debug_overlay(
                debug_systems.debug_ui_system.get_overlay(sid_joysticks),
                real_axes,
                joy_mask,
            );

            // Only record replay data if we're not already playing back a replay.
            if debug_systems.replay_recording_system.is_recording()
                && game_state.input_provider.is_realtime_player_input()
            {
                let record_replay_data = game_state
                    .debug_cvars
                    .record_replay
                    .read(&engine_state.config);
                if record_replay_data {
                    debug_systems
                        .replay_recording_system
                        .update(raw_events, real_axes, joy_mask);
                }
            }
        }

        {
            trace!("state_mgr::handle_actions", tracer);
            if game_state.state_mgr.handle_actions(
                &actions,
                engine_state,
                &mut game_state.gameplay_system,
            ) {
                engine_state.should_close = true;
                return Ok(false);
            }
        }

        // Update game systems
        {
            trace!("game_update", tracer);

            let axes = engine_state.systems.input_system.get_virtual_axes();
            #[cfg(feature = "prof_gameplay")]
            let gameplay_start_t = std::time::Instant::now();
            #[cfg(feature = "prof_gameplay")]
            let mut n_gameplay_updates = 0;

            let gameplay_system = &mut game_state.gameplay_system;

            gameplay_system.realtime_update(
                &real_dt,
                &engine_state.time,
                &actions,
                axes,
                &engine_state.config,
                clone_tracer!(tracer),
            );
            // @Robustness: limit or control somehow the number of updates done here
            // (maybe skip some frames if needed), or we can have a kind of "positive feedback"
            // where we keep losing time while attempting to update the game too many times
            // in the same frame.
            while game_state.execution_time > update_time {
                gameplay_system.update(
                    &update_time,
                    &engine_state.time,
                    &actions,
                    axes,
                    &engine_state.config,
                    clone_tracer!(tracer),
                );
                game_state.execution_time -= update_time;
                #[cfg(feature = "prof_gameplay")]
                {
                    n_gameplay_updates += 1;
                }
            }

            #[cfg(feature = "prof_gameplay")]
            println!(
                "[prof_gameplay] gameplay update took {} ms ({} updates, avg = {})",
                gameplay_start_t.elapsed().as_millis(),
                n_gameplay_updates,
                gameplay_start_t.elapsed().as_millis() / n_gameplay_updates,
            );
        }
    }

    // Update collisions
    {
        #[cfg(debug_assertions)]
        {
            let draw_entities = game_state
                .debug_cvars
                .draw_entities
                .read(&engine_state.config);
            let draw_velocities = game_state
                .debug_cvars
                .draw_entities_velocities
                .read(&engine_state.config);
            if draw_entities {
                debug_draw_entities(
                    &mut engine_state.debug_systems.debug_painter,
                    &game_state.gameplay_system.ecs_world,
                    draw_velocities,
                );
            }

            let draw_colliders = game_state
                .debug_cvars
                .draw_colliders
                .read(&engine_state.config);
            if draw_colliders {
                debug_draw_colliders(
                    &mut engine_state.debug_systems.debug_painter,
                    &game_state.gameplay_system.ecs_world,
                );
            }

            let draw_collision_quadtree = game_state
                .debug_cvars
                .draw_collision_quadtree
                .read(&engine_state.config);
            if draw_collision_quadtree {
                engine_state
                    .systems
                    .collision_system
                    .debug_draw_quadtree(&mut engine_state.debug_systems.debug_painter);
            }
        }

        trace!("collision_system::update", tracer);

        engine_state.systems.collision_system.update(
            &mut game_state.gameplay_system.ecs_world,
            clone_tracer!(tracer),
        );
    }

    // Update audio
    {
        trace!("audio_system_update", tracer);
        engine_state.systems.audio_system.update();
    }

    #[cfg(debug_assertions)]
    {
        let debug_systems = &mut engine_state.debug_systems;
        update_fps_debug_overlay(
            debug_systems.debug_ui_system.get_overlay(sid_fps),
            &game_state.fps_debug,
        );
        update_entities_debug_overlay(
            debug_systems.debug_ui_system.get_overlay(sid_entities),
            &game_state.gameplay_system.ecs_world,
        );
        update_camera_debug_overlay(
            debug_systems.debug_ui_system.get_overlay(sid_camera),
            &game_state.gameplay_system.get_camera(),
        );

        // Debug grid
        if game_state
            .debug_cvars
            .draw_debug_grid
            .read(&engine_state.config)
        {
            let square_size = game_state
                .debug_cvars
                .debug_grid_square_size
                .read(&engine_state.config);
            let opacity = game_state
                .debug_cvars
                .debug_grid_opacity
                .read(&engine_state.config);
            debug_draw_grid(
                &mut engine_state.debug_systems.debug_painter,
                &game_state.gameplay_system.get_camera().transform,
                engine_state.app_config.target_win_size,
                square_size,
                opacity as u8,
            );
        }
    }

    update_graphics(
        game_state,
        &mut game_resources.gfx,
        real_dt,
        time::duration_ratio(&game_state.execution_time, &update_time) as f32,
    )?;

    #[cfg(debug_assertions)]
    {
        let sleep = game_state
            .debug_cvars
            .extra_frame_sleep_ms
            .read(&game_state.engine_state.config) as u64;
        std::thread::sleep(Duration::from_millis(sleep));

        game_state.engine_state.config.update();
        game_state.fps_debug.tick(&real_dt);
    }

    Ok(true)
}

fn update_graphics(
    game_state: &mut Game_State,
    gres: &mut Gfx_Resources,
    real_dt: Duration,
    frame_lag_normalized: f32,
) -> Maybe_Error {
    trace!("update_graphics", game_state.engine_state.tracer);

    let window = &mut game_state.window;
    let cfg = &game_state.engine_state.config;
    gfx::window::set_clear_color(window, colors::rgb(0, 0, 0));
    gfx::window::clear(window);
    let render_cfg = render_system::Render_System_Config {
        clear_color: colors::color_from_hex(game_state.clear_color.read(cfg) as u32),
        smooth_by_extrapolating_velocity: game_state.smooth_by_extrapolating_velocity.read(cfg),
        #[cfg(debug_assertions)]
        draw_sprites_bg: game_state.debug_cvars.draw_sprites_bg.read(cfg),
        #[cfg(debug_assertions)]
        draw_sprites_bg_color: colors::color_from_hex(
            game_state.debug_cvars.draw_sprites_bg_color.read(cfg) as u32,
        ),
    };
    let render_args = render_system::Render_System_Update_Args {
        window,
        resources: gres,
        camera: game_state.gameplay_system.get_camera(),
        renderables: game_state.gameplay_system.get_renderable_entities(),
        ecs_world: &game_state.gameplay_system.ecs_world,
        frame_lag_normalized,
        cfg: render_cfg,
        tracer: clone_tracer!(game_state.engine_state.tracer),
    };

    render_system::update(render_args);

    #[cfg(debug_assertions)]
    {
        // Draw debug painter
        {
            game_state.engine_state.debug_systems.debug_painter.draw(
                window,
                gres,
                &game_state.gameplay_system.get_camera().transform,
                clone_tracer!(game_state.engine_state.tracer),
            );
            game_state.engine_state.debug_systems.debug_painter.clear();
        }

        // Draw debug UI
        {
            trace!("debug_ui_system::update", game_state.engine_state.tracer);
            game_state
                .engine_state
                .debug_systems
                .debug_ui_system
                .update(&real_dt, window, gres);
        }
    }

    {
        trace!("vsync", game_state.engine_state.tracer);
        gfx::window::display(window);
    }

    Ok(())
}

#[cfg(debug_assertions)]
fn update_joystick_debug_overlay(
    debug_overlay: &mut debug::overlay::Debug_Overlay,
    real_axes: &[input::joystick_mgr::Real_Axes_Values;
         input::bindings::joystick::JOY_COUNT as usize],
    joy_mask: u8,
) {
    use input::bindings::joystick;
    use std::convert::TryInto;

    debug_overlay.clear();

    for (joy_id, axes) in real_axes.iter().enumerate() {
        if (joy_mask & (1 << joy_id)) != 0 {
            debug_overlay.add_line_color(&format!("> Joy {} <", joy_id), colors::rgb(235, 52, 216));

            for i in 0u8..joystick::Joystick_Axis::_Count as u8 {
                let axis: joystick::Joystick_Axis = i.try_into().unwrap_or_else(|err| {
                    panic!("Failed to convert {} to a valid Joystick_Axis: {}", i, err)
                });
                debug_overlay.add_line_color(
                    &format!("{:?}: {:.2}", axis, axes[i as usize]),
                    colors::rgb(255, 255, 0),
                );
            }
        }
    }
}

#[cfg(debug_assertions)]
fn update_time_debug_overlay(debug_overlay: &mut debug::overlay::Debug_Overlay, time: &time::Time) {
    use ecs_engine::core::time::to_secs_frac;

    debug_overlay.clear();

    debug_overlay.add_line_color(
        &format!(
            "[time] game: {:.2}, real: {:.2}, scale: {:.2}, paused: {}",
            to_secs_frac(&time.get_game_time()),
            to_secs_frac(&time.get_real_time()),
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
) {
    debug_overlay.clear();
    debug_overlay.add_line_color(
        &format!("FPS: {}", fps.get_fps() as u32),
        colors::rgba(180, 180, 180, 200),
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
        transform.set_rotation(cgmath::Rad(0.));
        transform.translate_v(collider.offset);

        match collider.shape {
            Collider_Shape::Rect { width, height } => {
                debug_painter.add_rect(
                    Vec2f::new(width, height),
                    &transform,
                    &Paint_Properties {
                        color: if collider.colliding {
                            colors::rgba(255, 0, 0, 100)
                        } else {
                            colors::rgba(255, 255, 0, 100)
                        },
                        ..Default::default()
                    },
                );
            }
        }
    }
}

#[cfg(debug_assertions)]
fn debug_draw_entities(
    debug_painter: &mut Debug_Painter,
    ecs_world: &ecs_engine::ecs::ecs_world::Ecs_World,
    draw_velocities: bool,
) {
    use ecs_engine::core::common::shapes::{Arrow, Circle};
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
            &Paint_Properties {
                color: colors::rgb(50, 100, 200),
                ..Default::default()
            },
        );

        debug_painter.add_text(
            &format!(
                "{:.2},{:.2}",
                transform.position().x,
                transform.position().y
            ),
            transform.position(),
            10,
            &Paint_Properties {
                color: colors::WHITE,
                border_thick: 1.,
                border_color: colors::BLACK,
                ..Default::default()
            },
        );

        if draw_velocities {
            debug_painter.add_arrow(
                Arrow {
                    center: transform.position(),
                    direction: spatial.velocity * 10.,
                    thickness: 2.,
                    arrow_size: 20.,
                },
                &Paint_Properties {
                    color: colors::rgb(100, 0, 120),
                    ..Default::default()
                },
            );
        }
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
            debug_painter.add_rect(
                Vec2f::new(square_size, square_size),
                &transf,
                &Paint_Properties {
                    color,
                    ..Default::default()
                },
            );
            let pos = transf.position();
            debug_painter.add_text(
                &format!("{},{}", pos.x, pos.y),
                pos + Vec2f::new(5., 5.),
                (square_size as i32 / 6).max(8) as u16,
                &Paint_Properties {
                    color,
                    ..Default::default()
                },
            );
        }
    }
}
