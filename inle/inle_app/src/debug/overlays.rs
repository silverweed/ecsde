use super::systems::Debug_Systems;
use inle_common::colors;
use inle_common::paint_props::Paint_Properties;
use inle_core::time;
use inle_debug::painter::Debug_Painter;
use inle_gfx::render_window::{self, Render_Window_Handle};
use inle_input::mouse;
use inle_math::shapes::{Arrow, Circle, Line};
use inle_math::transform::Transform2D;
use inle_math::vector::{Vec2f, Vec2i};
use inle_win::window;
use inle_win::window::Camera;
use std::convert::TryInto;

// @Refactoring: maybe we should not have a monolithic function on the engine side,
// but only expose the various sub-functions and have the game call them.
pub fn update_debug(
    cvars: &crate::app::Engine_CVars,
    config: &inle_cfg::Config,
    window: &mut Render_Window_Handle,
    debug_systems: &mut Debug_Systems,
    time: &time::Time,
    fps_counter: &inle_debug::fps::Fps_Counter,
    input_state: &inle_input::input_state::Input_State,
    phys_world: &inle_physics::phys_world::Physics_World,
    camera: &Camera,
    collision_data: &inle_physics::physics::Collision_System_Debug_Data,
    lights: &inle_gfx::light::Lights,
) {
    // Overlays
    let display_overlays = cvars.debug.display_overlays.read(config);
    let overlays_were_visible = debug_systems.debug_ui.is_overlay_enabled(sid!("time"));
    if display_overlays {
        if !overlays_were_visible {
            set_debug_hud_enabled(&mut debug_systems.debug_ui, true);
        }

        update_time_debug_overlay(debug_systems.debug_ui.get_overlay(sid!("time")), time);

        update_fps_debug_overlay(
            debug_systems.debug_ui.get_overlay(sid!("fps")),
            fps_counter,
            (1000. / cvars.gameplay_update_tick_ms.read(config)) as u64,
            cvars.vsync.read(config),
        );

        update_win_debug_overlay(debug_systems.debug_ui.get_overlay(sid!("window")), window);
        update_physics_debug_overlay(
            debug_systems.debug_ui.get_overlay(sid!("physics")),
            collision_data,
        );
    } else if overlays_were_visible {
        set_debug_hud_enabled(&mut debug_systems.debug_ui, false);
    }

    update_joystick_debug_overlay(
        debug_systems.debug_ui.get_overlay(sid!("joysticks")),
        &input_state.raw.joy_state,
        config,
    );

    let draw_mouse_rulers = cvars.debug.draw_mouse_rulers.read(config);
    // NOTE: this must be always cleared or the mouse position will remain after enabling and disabling the cfg var
    debug_systems.debug_ui.get_overlay(sid!("mouse")).clear();
    if draw_mouse_rulers {
        let painter = &mut debug_systems.global_painter;
        update_mouse_debug_overlay(
            debug_systems.debug_ui.get_overlay(sid!("mouse")),
            painter,
            window,
            camera,
            input_state,
        );
    }

    /*
    let display_log_window = cvars
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

    let draw_lights = cvars.debug.draw_lights.read(config);
    if draw_lights {
        debug_draw_lights(&mut debug_systems.global_painter, lights);
    }

    /*
    let draw_particle_emitters = cvars.debug.draw_particle_emitters.read(config);
    if draw_particle_emitters {
        debug_draw_particles(&mut debug_systems.global_painter, particles);
    }
    */

    let draw_fps_graph = cvars.debug.draw_fps_graph.read(config);
    debug_systems
        .debug_ui
        .set_graph_enabled(sid!("fps"), draw_fps_graph);
    if draw_fps_graph {
        update_graph_fps(
            debug_systems.debug_ui.get_graph(sid!("fps")),
            time,
            fps_counter,
        );
    }

    let draw_prev_frame_t_graph = cvars.debug.draw_prev_frame_t_graph.read(config);
    debug_systems
        .debug_ui
        .set_graph_enabled(sid!("prev_frame_time"), draw_prev_frame_t_graph);
    if draw_prev_frame_t_graph {
        update_graph_prev_frame_t(
            debug_systems.debug_ui.get_graph(sid!("prev_frame_time")),
            time,
        );
    }

    let draw_grid = cvars.debug.draw_debug_grid.read(config);
    if draw_grid {
        let square_size = cvars.debug.debug_grid_square_size.read(config);
        let opacity = cvars.debug.debug_grid_opacity.read(config);
        let font_size = cvars.debug.debug_grid_font_size.read(config);
        let win_size = window::get_window_real_size(window);
        debug_draw_grid(
            &mut debug_systems.global_painter,
            &camera.transform,
            win_size,
            square_size,
            opacity as _,
            font_size as _,
        );
    }

    let draw_colliders = cvars.debug.draw_colliders.read(config);
    if draw_colliders {
        debug_draw_colliders(&mut debug_systems.global_painter, phys_world);
    }
    let draw_velocities = cvars.debug.draw_velocities.read(config);
    if draw_velocities {
        debug_draw_velocities(&mut debug_systems.global_painter, phys_world);
    }

    // @Cleanup
    if cvars.debug.draw_buf_alloc.read(config) {
        inle_debug::backend_specific_debugs::draw_backend_specific_debug(
            window,
            &mut debug_systems.global_painter,
        );
    }
}

fn update_joystick_debug_overlay(
    debug_overlay: &mut inle_debug::overlay::Debug_Overlay,
    joy_state: &inle_input::joystick::Joystick_State,
    cfg: &inle_cfg::Config,
) {
    use inle_input::joystick;

    debug_overlay.clear();

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
                let deadzone = joy_state.joysticks[joy_id]
                    .as_ref()
                    .unwrap()
                    .config
                    .deadzone
                    .read(cfg);
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

fn update_time_debug_overlay(
    debug_overlay: &mut inle_debug::overlay::Debug_Overlay,
    time: &time::Time,
) {
    debug_overlay.clear();

    debug_overlay
        .add_line(&format!(
            "[time] game: {:.2}, real: {:.2}, scale: {:.2}, paused: {}",
            time.game_time().as_secs_f32(),
            time.real_time().as_secs_f32(),
            time.time_scale,
            if time.paused { "yes" } else { "no" },
        ))
        .with_color(colors::rgb(100, 200, 200));
}

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

fn update_mouse_debug_overlay(
    debug_overlay: &mut inle_debug::overlay::Debug_Overlay,
    painter: &mut Debug_Painter,
    window: &Render_Window_Handle,
    camera: &Camera,
    input_state: &inle_input::input_state::Input_State,
) {
    let (win_w, win_h) = {
        let (win_w, win_h) = window::get_window_target_size(window);
        (win_w as i32, win_h as i32)
    };
    let raw_pos = mouse::raw_mouse_pos(&input_state.raw.mouse_state);
    let pos = window::correct_mouse_pos_in_window(window, raw_pos);

    // Adjust overlay pos
    debug_overlay.position = Vec2f::from(pos) + v2!(5., -15.);
    let overlay_size = debug_overlay.bounds().size();
    debug_overlay.position.x =
        inle_math::math::clamp(debug_overlay.position.x, 0., win_w as f32 - overlay_size.x);
    debug_overlay.position.y =
        inle_math::math::clamp(debug_overlay.position.y, overlay_size.y, win_h as f32);
    debug_overlay
        .add_line(&format!("s {},{}", pos.x, pos.y))
        .with_color(colors::rgba(220, 220, 220, 220));

    // Get world position
    let wpos = render_window::unproject_screen_pos(Vec2i::from(raw_pos), window, camera);
    debug_overlay
        .add_line(&format!("w {:.2},{:.2}", wpos.x, wpos.y))
        .with_color(colors::rgba(200, 200, 200, 220));

    let from_horiz = v2!(0., pos.y as f32);
    let to_horiz = v2!(win_w as f32, pos.y as f32);
    let from_vert = v2!(pos.x as f32, 0.);
    let to_vert = v2!(pos.x as f32, win_h as f32);

    let color = colors::rgba(255, 255, 255, 150);
    painter.add_line_ss(
        Line {
            from: from_horiz,
            to: to_horiz,
            thickness: 2.0,
        },
        color,
    );
    painter.add_line_ss(
        Line {
            from: from_vert,
            to: to_vert,
            thickness: 2.0,
        },
        color,
    );
}

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

fn debug_draw_lights(
    //screenspace_debug_painter: &mut Debug_Painter,
    debug_painter: &mut Debug_Painter,
    lights: &inle_gfx::light::Lights,
) {
    /*screenspace_*/debug_painter.add_shaded_text(
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

fn update_graph_fps(
    graph: &mut inle_debug::graph::Debug_Graph_View,
    time: &time::Time,
    fps: &inle_debug::fps::Fps_Counter,
) {
    const TIME_LIMIT: f32 = 60.0;

    let fps = fps.get_instant_fps();
    inle_debug::graph::add_point_and_scroll(graph, time.real_time(), TIME_LIMIT, fps);
}

fn update_graph_prev_frame_t(graph: &mut inle_debug::graph::Debug_Graph_View, time: &time::Time) {
    const TIME_LIMIT: f32 = 10.0;

    inle_debug::graph::add_point_and_scroll(
        graph,
        time.real_time(),
        TIME_LIMIT,
        time.prev_frame_time().as_secs_f32() * 1000.,
    );
}

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

fn debug_draw_colliders(
    debug_painter: &mut Debug_Painter,
    phys_world: &inle_physics::phys_world::Physics_World,
) {
    use inle_physics::collider::Collision_Shape;

    for collider in phys_world.get_all_colliders() {
        // Note: since our collision detector doesn't handle rotation, draw the colliders with rot = 0
        // @Incomplete: scale?
        let mut transform = Transform2D::from_pos_rot_scale(
            collider.position,
            inle_math::angle::rad(0.),
            v2!(1., 1.),
        );

        debug_painter.add_text(
            &format!("lr {}", collider.layer),
            collider.position,
            10,
            colors::BLACK,
        );

        let mut cld_color = colors::rgba(255, 255, 0, 100);

        let colliding_with = phys_world.get_collisions(collider.handle);
        if !colliding_with.is_empty() {
            cld_color = colors::rgba(255, 0, 0, 100);
        }

        for cls_data in colliding_with {
            let oth_cld = phys_world.get_collider(cls_data.other_collider).unwrap();
            debug_painter.add_arrow(
                Arrow {
                    center: collider.position,
                    direction: oth_cld.position - collider.position,
                    thickness: 1.,
                    arrow_size: 5.,
                },
                colors::GREEN,
            );
            debug_painter.add_arrow(
                Arrow {
                    center: collider.position,
                    direction: cls_data.info.normal * 20.0,
                    thickness: 1.,
                    arrow_size: 5.,
                },
                colors::PINK,
            );
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
            colors::ORANGE,
        );

        debug_painter.add_text(
            &format!("{},{}", collider.handle.gen, collider.handle.index),
            collider.position + v2!(2., -3.),
            10,
            colors::ORANGE,
        );
    }
}

fn debug_draw_velocities(
    debug_painter: &mut Debug_Painter,
    phys_world: &inle_physics::phys_world::Physics_World,
) {
    for collider in phys_world.get_all_colliders() {
        let mut v = collider.velocity;
        let len = v.magnitude().min(200.);
        v = v.normalized_or_zero() * len;
        if len > f32::EPSILON {
            debug_painter.add_arrow(
                Arrow {
                    center: collider.position,
                    direction: v,
                    thickness: 2.,
                    arrow_size: 15.,
                },
                colors::DARK_RED,
            );
            if v.x.abs() > f32::EPSILON {
                debug_painter.add_arrow(
                    Arrow {
                        center: collider.position,
                        direction: v2!(v.x, 0.),
                        thickness: 1.,
                        arrow_size: 10.,
                    },
                    colors::DARK_GREEN,
                );
                debug_painter.add_text(
                    &format!("{:.3}", collider.velocity.x),
                    collider.position + v2!(4., 10.),
                    10,
                    colors::DARK_GREEN,
                );
            }
            if v.y.abs() > f32::EPSILON {
                debug_painter.add_arrow(
                    Arrow {
                        center: collider.position,
                        direction: v2!(0., v.y),
                        thickness: 1.,
                        arrow_size: 10.,
                    },
                    colors::DARK_GREEN,
                );
                debug_painter.add_text(
                    &format!("{:.3}", collider.velocity.y),
                    collider.position + v2!(4., 20.),
                    10,
                    colors::DARK_GREEN,
                );
            }
        } else {
            debug_painter.add_circle(
                Circle {
                    center: collider.position,
                    radius: 2.,
                },
                colors::DARK_RED,
            );
        }

        debug_painter.add_text(
            &format!("{:.3}", collider.velocity.magnitude()),
            collider.position + v2!(2., -3.),
            12,
            colors::DARK_RED,
        );
    }
}
