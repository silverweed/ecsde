use crate::debug::systems::position_history_system::C_Position_History;
use crate::systems::ground_detection_system::C_Ground_Detection;
use inle_app::render_system;
use inle_common::colors;
use inle_common::paint_props::Paint_Properties;
use inle_core::time;
use inle_debug::painter::Debug_Painter;
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_query_new::Ecs_Query;
use inle_ecs::ecs_world::{Component_Manager, Component_Updates, Ecs_World, Entity};
use inle_gfx::components::C_Renderable;
use inle_gfx::render_window::{self, Render_Window_Handle};
use inle_input::input_state::Input_State;
use inle_input::mouse;
use inle_math::angle::rad;
use inle_math::shapes::{Arrow, Circle, Line};
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use inle_physics::collider::{C_Collider, Collision_Shape};
use inle_physics::phys_world::Physics_World;
use inle_physics::physics;
use inle_win::window;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::time::Duration;

pub struct Debug_Ecs_Queries {
    pub draw_colliders: Ecs_Query,
    pub just_spatials: Ecs_Query,
    pub draw_component_lists: Ecs_Query,
    pub draw_entities_prev_frame_ghost: Ecs_Query,
    pub draw_entities_pos_history: Ecs_Query,
    pub draw_entities_touching_ground: Ecs_Query,
}

impl Debug_Ecs_Queries {
    pub fn update_queries(
        &mut self,
        comp_updates: &HashMap<Entity, Component_Updates>,
        comp_mgr: &Component_Manager,
    ) {
        for (entity, updates) in comp_updates {
            let up = |qry: &mut Ecs_Query| {
                qry.update(comp_mgr, *entity, &updates.added, &updates.removed);
            };

            up(&mut self.draw_colliders);
            up(&mut self.just_spatials);
            up(&mut self.draw_component_lists);
            up(&mut self.draw_entities_prev_frame_ghost);
            up(&mut self.draw_entities_pos_history);
            up(&mut self.draw_entities_touching_ground);
        }
    }
}

pub fn create_debug_ecs_queries() -> Debug_Ecs_Queries {
    Debug_Ecs_Queries {
        draw_colliders: Ecs_Query::default()
            .require::<C_Collider>()
            .require::<C_Spatial2D>(),
        just_spatials: Ecs_Query::default().require::<C_Spatial2D>(),
        draw_component_lists: Ecs_Query::default(),
        draw_entities_prev_frame_ghost: Ecs_Query::default()
            .require::<C_Spatial2D>()
            .require::<C_Renderable>(),
        draw_entities_pos_history: Ecs_Query::default().require::<C_Position_History>(),
        draw_entities_touching_ground: Ecs_Query::default()
            .require::<C_Spatial2D>()
            .require::<C_Ground_Detection>(),
    }
}

pub fn update_joystick_debug_overlay(
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

pub fn update_time_debug_overlay(
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

pub fn update_fps_debug_overlay(
    debug_overlay: &mut inle_debug::overlay::Debug_Overlay,
    fps: &inle_debug::fps::Fps_Counter,
    target_fps: u64,
    vsync: bool,
    prev_frame_t: &Duration,
) {
    debug_overlay.clear();
    debug_overlay
        .add_line(&format!(
            "FPS: {} (target ~{}, vsync {}) | {:.2} ms",
            fps.get_fps() as u32,
            target_fps,
            if vsync { "on" } else { "off" },
            prev_frame_t.as_secs_f32() * 1000.,
        ))
        .with_color(colors::rgba(180, 180, 180, 200));
}

pub fn update_mouse_debug_overlay(
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

pub fn update_win_debug_overlay(
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

pub fn update_entities_and_draw_calls_debug_overlay(
    debug_overlay: &mut inle_debug::overlay::Debug_Overlay,
    ecs_world: &Ecs_World,
    window: &Render_Window_Handle,
) {
    debug_overlay.clear();
    debug_overlay
        .add_line(&format!(
            "Entities: {} | draw calls: {}",
            ecs_world.entities().len(),
            inle_gfx::render_window::n_draw_calls_prev_frame(window)
        ))
        .with_color(colors::rgba(220, 100, 180, 220));
}

pub fn update_camera_debug_overlay(
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

pub fn update_physics_debug_overlay(
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

pub fn update_record_debug_overlay(
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

pub fn debug_draw_colliders(
    debug_painter: &mut Debug_Painter,
    query: &Ecs_Query,
    ecs_world: &Ecs_World,
    phys_world: &Physics_World,
) {
    use crate::collisions::Game_Collision_Layer;

    foreach_entity!(query, ecs_world,
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

pub fn debug_draw_transforms(
    debug_painter: &mut Debug_Painter,
    query: &Ecs_Query,
    ecs_world: &Ecs_World,
    window: &Render_Window_Handle,
    input_state: &Input_State,
    camera: &Transform2D,
) {
    let mpos = render_window::mouse_pos_in_world(window, &input_state.raw.mouse_state, camera);
    let mut entity_overlapped = (Entity::INVALID, 0.);
    foreach_entity!(query, ecs_world,
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

pub fn debug_draw_velocities(
    debug_painter: &mut Debug_Painter,
    query: &Ecs_Query,
    ecs_world: &Ecs_World,
) {
    const COLOR: colors::Color = colors::rgb(100, 0, 120);

    foreach_entity!(query, ecs_world,
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

pub fn debug_draw_component_lists(
    debug_painter: &mut Debug_Painter,
    query: &Ecs_Query,
    ecs_world: &Ecs_World,
) {
    use crate::debug::entity_debug::C_Debug_Data;

    foreach_entity!(query, ecs_world,
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

pub fn debug_draw_lights(
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

pub fn debug_draw_entities_prev_frame_ghost(
    window: &mut Render_Window_Handle,
    batches: &mut inle_gfx::render::batcher::Batches,
    shader_cache: &mut inle_resources::gfx::Shader_Cache,
    env: &inle_core::env::Env_Info,
    query: &Ecs_Query,
    ecs_world: &mut Ecs_World,
    is_paused: bool,
) {
    trace!("debug_draw_entities_prev_frame_ghost");

    use crate::debug::entity_debug::C_Debug_Data;
    //use crate::systems::pixel_collision_system::C_Texture_Collider;
    use crate::gfx::shaders::SHD_SPRITE_UNLIT;
    use inle_gfx::render;
    use inle_resources::gfx::shader_path;

    let unlit_shader = shader_cache.load_shader(&shader_path(env, SHD_SPRITE_UNLIT));

    foreach_entity!(query, ecs_world,
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

pub fn debug_draw_entities_pos_history(
    painter: &mut Debug_Painter,
    query: &Ecs_Query,
    ecs_world: &Ecs_World,
) {
    use inle_math::math::lerp;

    foreach_entity!(query, ecs_world,
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
pub fn debug_draw_grid(
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

pub fn update_graph_fps(
    graph: &mut inle_debug::graph::Debug_Graph_View,
    time: &time::Time,
    fps: &inle_debug::fps::Fps_Counter,
) {
    const TIME_LIMIT: f32 = 60.0;

    let fps = fps.get_instant_fps();
    inle_debug::graph::add_point_and_scroll(graph, time.real_time(), TIME_LIMIT, fps);
}

pub fn update_graph_prev_frame_t(
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

pub fn get_render_system_debug_visualization(
    debug_cvars: &crate::game_state::Debug_CVars,
    cfg: &inle_cfg::Config,
) -> render_system::Debug_Visualization {
    match debug_cvars.render_debug_visualization.read(cfg).as_str() {
        "1" | "b" | "bounds" => render_system::Debug_Visualization::Sprites_Boundaries,
        "2" | "n" | "normals" => render_system::Debug_Visualization::Normals,
        "3" | "m" | "materials" => render_system::Debug_Visualization::Materials,
        _ => render_system::Debug_Visualization::None,
    }
}

pub fn set_debug_hud_enabled(debug_ui: &mut inle_debug::debug_ui::Debug_Ui_System, enabled: bool) {
    debug_ui.set_overlay_enabled(sid!("time"), enabled);
    debug_ui.set_overlay_enabled(sid!("fps"), enabled);
    debug_ui.set_overlay_enabled(sid!("window"), enabled);
    debug_ui.set_overlay_enabled(sid!("entities"), enabled);
    debug_ui.set_overlay_enabled(sid!("camera"), enabled);
    debug_ui.set_overlay_enabled(sid!("physics"), enabled);
    debug_ui.set_overlay_enabled(sid!("joysticks"), enabled);
    debug_ui.frame_scroller.hidden = !enabled;
}

pub fn debug_draw_particle_emitters(
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

pub fn debug_draw_entities_touching_ground(
    debug_painter: &mut Debug_Painter,
    query: &Ecs_Query,
    ecs_world: &Ecs_World,
) {
    foreach_entity!(query, ecs_world,
        read: C_Spatial2D, C_Ground_Detection;
        write: ;
        |_e, (spatial, gnd_detect): (&C_Spatial2D, &C_Ground_Detection), ()| {
            debug_painter.add_text(
                &format!("touching ground: {}", gnd_detect.touching_ground),
                spatial.transform.position() + v2!(5., 0.),
                5,
                if gnd_detect.touching_ground { colors::GREEN } else { colors::RED });
            debug_painter.add_text(
                &format!("just touched: {}", gnd_detect.just_touched_ground),
                spatial.transform.position() + v2!(5., 4.),
                5,
                if gnd_detect.just_touched_ground { colors::GREEN } else { colors::RED });
            debug_painter.add_text(
                &format!("just left: {}", gnd_detect.just_left_ground),
                spatial.transform.position() + v2!(5., 8.),
                5,
                if gnd_detect.just_left_ground { colors::GREEN } else { colors::RED });
    });
}
