use crate::{Game_State, Game_Resources};
use std::time::Duration;
use inle_common::stringid::String_Id;
use inle_core::time;
use inle_common::paint_props::Paint_Properties;
use std::convert::TryInto;
use std::collections::HashMap;
use inle_math::vector::{Vec2i,Vec2f};
use inle_math::transform::Transform2D;
use inle_math::shapes::{Circle, Arrow, Line};
use inle_gfx::render_window::{self, Render_Window_Handle};
use inle_input::mouse;
use inle_win::window;
use inle_common::colors;
use inle_debug::painter::Debug_Painter;

pub fn update_debug(
    game_state: &mut Game_State,
    game_resources: &mut Game_Resources,
) {
    let collision_debug_data: HashMap<String_Id, inle_physics::physics::Collision_System_Debug_Data> = HashMap::default();
    let dt = game_state.time.dt();
    let debug_systems = &mut game_state.debug_systems;
    let cvars = &game_state.cvars;
    let dbg_cvars = &game_state.debug_cvars;
    let config = &game_state.config;

    // Overlays
    let display_overlays = dbg_cvars
        .display_overlays
        .read(config);
    let overlays_were_visible = debug_systems.debug_ui.is_overlay_enabled(sid!("time"));
    if display_overlays {
        if !overlays_were_visible {
            set_debug_hud_enabled(&mut debug_systems.debug_ui, true);
        }

        update_time_debug_overlay(
            debug_systems.debug_ui.get_overlay(sid!("time")),
            &game_state.time,
            1
        );

        update_fps_debug_overlay(
            debug_systems.debug_ui.get_overlay(sid!("fps")),
            &game_state.fps_counter,
            (1000.
                / cvars
                    .gameplay_update_tick_ms
                    .read(config)) as u64,
            cvars.vsync.read(config),
        );

        update_win_debug_overlay(
            debug_systems.debug_ui.get_overlay(sid!("window")),
            &game_state.window,
        );
    } else if overlays_were_visible {
        set_debug_hud_enabled(&mut debug_systems.debug_ui, false);
    }

    let input_state = &game_state.input;
    let draw_mouse_rulers = dbg_cvars
        .draw_mouse_rulers
        .read(config);
    // NOTE: this must be always cleared or the mouse position will remain after enabling and disabling the cfg var
    debug_systems.debug_ui.get_overlay(sid!("mouse")).clear();
    if draw_mouse_rulers {
        let painter = &mut debug_systems.global_painter;
        update_mouse_debug_overlay(
            debug_systems.debug_ui.get_overlay(sid!("mouse")),
            painter,
            &game_state.window,
            None,
            input_state,
        );
    }

    /*
    let display_log_window = dbg_cvars
        .display_log_window
        .read(config);
    let log_window_enabled = debug_systems
        .debug_ui
        .is_log_window_enabled(sid!("log_window"));
    if display_log_window != log_window_enabled {
        debug_systems
            .debug_ui
            .set_log_window_enabled(sid!("log_window"), display_log_window);
    }
    */

    let draw_fps_graph = dbg_cvars
        .draw_fps_graph
        .read(config);
    debug_systems
        .debug_ui
        .set_graph_enabled(sid!("fps"), draw_fps_graph);
    if draw_fps_graph {
        update_graph_fps(
            debug_systems.debug_ui.get_graph(sid!("fps")),
            &game_state.time,
            &game_state.fps_counter,
        );
    }

    let draw_prev_frame_t_graph = dbg_cvars
        .draw_prev_frame_t_graph
        .read(config);
    debug_systems
        .debug_ui
        .set_graph_enabled(sid!("prev_frame_time"), draw_prev_frame_t_graph);
    if draw_prev_frame_t_graph {
        update_graph_prev_frame_t(
            debug_systems.debug_ui.get_graph(sid!("prev_frame_time")),
            &game_state.time,
            &game_state.prev_frame_time,
        );
    }

    ////// Per-Level debugs //////
    /*
    let painters = &mut debug_systems.painters;
    let debug_ui = &mut debug_systems.debug_ui;
    let target_win_size = game_state.app_config.target_win_size;

    let is_paused = game_state.time.paused && !game_state.time.is_stepping();
    let cvars = &game_state.debug_cvars;
    let config = &game_state.config;
    let draw_entities = cvars.draw_entities.read(config);
    let draw_component_lists = cvars.draw_component_lists.read(config);
    let draw_velocities = cvars.draw_velocities.read(config);
    let draw_entity_prev_frame_ghost = cvars
        .draw_entity_prev_frame_ghost
        .read(config);
    let draw_entity_pos_history = cvars.draw_entity_pos_history.read(config);
    let draw_colliders = cvars.draw_colliders.read(config);
    let draw_debug_grid = cvars.draw_debug_grid.read(config);
    let grid_square_size = cvars.debug_grid_square_size.read(config);
    let grid_font_size = cvars.debug_grid_font_size.read(config);
    let grid_opacity = cvars.debug_grid_opacity.read(config) as u8;
    let draw_world_chunks = cvars.draw_world_chunks.read(config);
    let draw_lights = cvars.draw_lights.read(config);
    let draw_particle_emitters = cvars.draw_particle_emitters.read(config);
    let global_painter = &mut debug_systems.global_painter;
    let window = &mut game_state.window;
    let shader_cache = &mut game_resources.shader_cache;
    let env = &game_state.env;
    let particle_mgrs = &engine_state.systems.particle_mgrs;

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
    let cfg = &game_state.config;
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
    */

    // @Cleanup
    if dbg_cvars.draw_buf_alloc.read(config) {
        inle_debug::backend_specific_debugs::draw_backend_specific_debug(&mut game_state.window, &mut game_state.debug_systems.global_painter);
    }
}

#[cfg(debug_assertions)]
fn update_joystick_debug_overlay(
    debug_overlay: &mut inle_debug::overlay::Debug_Overlay,
    joy_state: &inle_input::joystick::Joystick_State,
    input_cfg: crate::input::Input_Config,
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
    input_state: &inle_input::input_state::Input_State,
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
        let mpos = Vec2i::from(Vec2f::from(mouse::raw_mouse_pos(&input_state.raw.mouse_state)));
        let wpos = render_window::mouse_pos_in_world(window, mpos, &camera);
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
    collision_data: &inle_physics::physics::Collision_System_Debug_Data,
) {
    debug_overlay.clear();
    debug_overlay
        .add_line(&format!(
            "[phys] n_inter_tests: {}",
            collision_data.n_intersection_tests,
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

/*
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
*/

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
                color: pl.color, // @Incomplete: make this transparent when we can render borders
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
        ((cx - sw * 0.5) / square_size).floor() * square_size,
        ((cy - sh * 0.5) / square_size).floor() * square_size,
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
