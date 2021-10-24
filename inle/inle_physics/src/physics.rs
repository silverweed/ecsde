// reference: https://gamedevelopment.tutsplus.com/tutorials/how-to-create-a-custom-2d-physics-engine-the-basics-and-impulse-resolution--gamedev-6331

use super::layers::Collision_Matrix;
use super::phys_world::{
    Collider_Handle, Collision_Data, Collision_Info, Phys_Data, Physics_World,
};
use super::spatial::Spatial_Accelerator;
use crate::collider::{Collider, Collision_Shape};
use inle_alloc::temp::{excl_temp_array, Temp_Allocator};
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::{Ecs_World, Entity};
use inle_events::evt_register::{Event, Event_Register};
use inle_math::math::clamp;
use inle_math::vector::{sanity_check_v, Vec2f};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

type Rigidbodies = HashMap<Collider_Handle, Rigidbody>;

pub struct Evt_Collision_Happened;

impl Event for Evt_Collision_Happened {
    type Args = Collision_Data;
}

#[derive(Default)]
pub struct Physics_Settings {
    pub collision_matrix: Collision_Matrix,
}

#[derive(Debug, Clone)]
struct Rigidbody {
    pub shape: Collision_Shape,
    pub entity: Entity,
    pub position: Vec2f,
    pub offset: Vec2f,
    pub velocity: Vec2f,
    pub phys_data: Phys_Data,
}

#[derive(Debug, Clone)]
struct Collision_Info_Internal {
    cld1: Collider_Handle,
    cld2: Collider_Handle,
    info: Collision_Info,
}

#[cfg(debug_assertions)]
#[derive(Default)]
pub struct Collision_System_Debug_Data {
    // How many intersections were tested during this frame
    pub n_intersection_tests: usize,
}

fn detect_circle_circle(a: &Collider, b: &Collider) -> Option<Collision_Info_Internal> {
    trace!("physics::detect_circle_circle");

    let a_radius = if let Collision_Shape::Circle { radius } = a.shape {
        radius
    } else {
        panic!("Failed to unwrap Circle!")
    };
    let b_radius = if let Collision_Shape::Circle { radius } = b.shape {
        radius
    } else {
        panic!("Failed to unwrap Circle!")
    };

    let diff = b.position - a.position;
    let r = a_radius + b_radius;
    let rsquared = r * r;

    if diff.magnitude2() > rsquared {
        return None;
    }

    let dist = diff.magnitude();
    if dist > std::f32::EPSILON {
        Some(Collision_Info_Internal {
            cld1: a.handle,
            cld2: b.handle,
            info: Collision_Info {
                normal: diff / dist,
                penetration: r - dist,
            },
        })
    } else {
        // circles are in the same position
        Some(Collision_Info_Internal {
            cld1: a.handle,
            cld2: b.handle,
            info: Collision_Info {
                normal: v2!(1., 0.),   // Arbitrary
                penetration: a_radius, // Arbitrary
            },
        })
    }
}

fn detect_rect_rect(a: &Collider, b: &Collider) -> Option<Collision_Info_Internal> {
    trace!("physics::detect_rect_rect");

    const OVERLAP_EPSILON: f32 = f32::EPSILON;

    let (a_width, a_height) = if let Collision_Shape::Rect { width, height } = a.shape {
        (width, height)
    } else {
        panic!("Failed to unwrap Rect!")
    };
    let (b_width, b_height) = if let Collision_Shape::Rect { width, height } = b.shape {
        (width, height)
    } else {
        panic!("Failed to unwrap Rect!")
    };
    let diff = b.position - a.position;

    let a_half_ext_x = a_width * 0.5;
    let b_half_ext_x = b_width * 0.5;

    // Apply SAT on X axis
    let x_overlap = a_half_ext_x + b_half_ext_x - diff.x.abs();
    if x_overlap <= OVERLAP_EPSILON {
        return None;
    }

    let a_half_ext_y = a_height * 0.5;
    let b_half_ext_y = b_height * 0.5;

    // Apply SAT on Y axis
    let y_overlap = a_half_ext_y + b_half_ext_y - diff.y.abs();
    if y_overlap <= OVERLAP_EPSILON {
        return None;
    }

    // Find least penetration axis
    if x_overlap < y_overlap {
        let normal = if diff.x < 0. {
            v2!(-1., 0.)
        } else {
            v2!(1., 0.)
        };
        Some(Collision_Info_Internal {
            cld1: a.handle,
            cld2: b.handle,
            info: Collision_Info {
                normal,
                penetration: x_overlap,
            },
        })
    } else {
        let normal = if diff.y < 0. {
            v2!(0., -1.)
        } else {
            v2!(0., 1.)
        };
        Some(Collision_Info_Internal {
            cld1: a.handle,
            cld2: b.handle,
            info: Collision_Info {
                normal,
                penetration: y_overlap,
            },
        })
    }
}

#[allow(clippy::collapsible_if, clippy::collapsible_else_if)]
fn detect_circle_rect(circle: &Collider, rect: &Collider) -> Option<Collision_Info_Internal> {
    trace!("physics::detect_circle_rect");

    let (r_width, r_height) = if let Collision_Shape::Rect { width, height } = rect.shape {
        (width, height)
    } else {
        panic!("Failed to unwrap Rect!")
    };
    let c_radius = if let Collision_Shape::Circle { radius } = circle.shape {
        radius
    } else {
        panic!("Failed to unwrap Circle!")
    };

    let diff = circle.position - rect.position;
    let half_ext_x = r_width * 0.5;
    let half_ext_y = r_height * 0.5;

    let mut closest = v2!(
        clamp(diff.x, -half_ext_x, half_ext_x),
        clamp(diff.y, -half_ext_y, half_ext_y),
    );

    let mut inside = false;

    // @Audit! We want "closest == diff": is this a proper way to do that?
    if (closest - diff).magnitude2() <= std::f32::EPSILON {
        inside = true;
        if diff.x.abs() > diff.y.abs() {
            if closest.x > 0. {
                closest.x = half_ext_x;
            } else {
                closest.x = -half_ext_x;
            }
        } else {
            if closest.y > 0. {
                closest.y = half_ext_y;
            } else {
                closest.y = -half_ext_y;
            }
        }
    }

    let normal = diff - closest;
    let d = normal.magnitude2();
    let r = c_radius;

    if (d > r * r) && !inside {
        return None;
    }

    let d = d.sqrt();
    let normal = normal.normalized_or_zero();
    debug_assert!(normal.is_normalized(), "{}", normal.magnitude());

    Some(Collision_Info_Internal {
        cld1: circle.handle,
        cld2: rect.handle,
        info: Collision_Info {
            normal: if inside { normal } else { -normal },
            penetration: r - d,
        },
    })
}

fn detect_rect_circle(rect: &Collider, circle: &Collider) -> Option<Collision_Info_Internal> {
    detect_circle_rect(circle, rect)
}

fn collision_shape_type_index(shape: &Collision_Shape) -> usize {
    match shape {
        Collision_Shape::Circle { .. } => 0,
        Collision_Shape::Rect { .. } => 1,
    }
}

type Collision_Cb = fn(&Collider, &Collider) -> Option<Collision_Info_Internal>;

const COLLISION_CB_TABLE: [[Collision_Cb; 2]; 2] = [
    [detect_circle_circle, detect_circle_rect],
    [detect_rect_circle, detect_rect_rect],
];

fn detect_collisions<T_Spatial_Accelerator>(
    phys_world: &Physics_World,
    accelerator: &T_Spatial_Accelerator,
    collision_matrix: &Collision_Matrix,
    temp_alloc: &mut Temp_Allocator,
    #[cfg(debug_assertions)] debug_data: &mut Collision_System_Debug_Data,
) -> Vec<Collision_Info_Internal>
where
    T_Spatial_Accelerator: Spatial_Accelerator<Collider_Handle>,
{
    trace!("physics::detect_collisions");

    #[cfg(debug_assertions)]
    {
        debug_data.n_intersection_tests = 0;
    }

    let mut collision_infos = vec![];
    let mut storage: HashSet<(*const Collider, *const Collider)> = HashSet::default();

    // @Speed: maybe we should iterate on the chunks? Can we do that in parallel?
    for a in phys_world.colliders.iter().filter(|cld| !cld.is_static) {
        let a_extent = a.shape.extent();
        let a_shape = collision_shape_type_index(&a.shape);
        let a_partial_cb = COLLISION_CB_TABLE[a_shape];
        let ent_a = a.entity;

        let mut neighbours = excl_temp_array(temp_alloc);
        accelerator.get_neighbours(a.position, a_extent, &mut neighbours);

        for &b_handle in &neighbours {
            let b = phys_world.get_collider(b_handle).unwrap();
            let ent_b = b.entity;
            if ent_a == ent_b {
                continue;
            }
            let b_shape = collision_shape_type_index(&b.shape);

            let pa: *const Collider = a as *const _;
            let pb: *const Collider = b as *const _;

            if !collision_matrix.layers_collide(a.layer, b.layer) || storage.contains(&(pa, pb)) {
                continue;
            }

            let info = a_partial_cb[b_shape](a, b);

            #[cfg(debug_assertions)]
            {
                debug_data.n_intersection_tests += 1;
            }

            if let Some(info) = info {
                collision_infos.push(info);
                storage.insert((pa, pb));
            }
        }
    }

    collision_infos
}

fn solve_collision_velocities(
    objects: &mut Rigidbodies,
    a_idx: Collider_Handle,
    b_idx: Collider_Handle,
    normal: Vec2f,
) {
    trace!("physics::solve_collisions_velocities");

    let a = objects[&a_idx].clone();
    let b = objects[&b_idx].clone();

    if a.phys_data.inv_mass + b.phys_data.inv_mass == 0. {
        // Both infinite-mass objects
        return;
    }

    let rel_vel = b.velocity - a.velocity;
    let vel_along_normal = rel_vel.dot(normal);

    if vel_along_normal > 0. {
        return;
    }

    sanity_check_v(a.velocity);
    sanity_check_v(b.velocity);
    sanity_check_v(rel_vel);
    debug_assert!(!vel_along_normal.is_nan());

    let e = a.phys_data.restitution.min(b.phys_data.restitution);

    // Impulse scalar
    let j = -(1. + e) * vel_along_normal / (a.phys_data.inv_mass + b.phys_data.inv_mass);
    debug_assert!(!j.is_nan());

    let impulse = j * normal;
    objects.get_mut(&a_idx).unwrap().velocity -= 1. * a.phys_data.inv_mass * impulse;
    objects.get_mut(&b_idx).unwrap().velocity += 1. * b.phys_data.inv_mass * impulse;

    // @Speed: cloning
    let a = objects[&a_idx].clone();
    let b = objects[&b_idx].clone();

    // apply friction
    let new_rel_vel = b.velocity - a.velocity;
    sanity_check_v(a.velocity);
    sanity_check_v(b.velocity);
    sanity_check_v(new_rel_vel);

    let tangent = (new_rel_vel - new_rel_vel.dot(normal) * normal).normalized_or_zero();

    let jt = -new_rel_vel.dot(tangent) / (a.phys_data.inv_mass * b.phys_data.inv_mass);

    let mu = (a.phys_data.static_friction + b.phys_data.static_friction) * 0.5;

    let friction_impulse = if jt.abs() < j * mu {
        jt * tangent
    } else {
        let dyn_friction = (a.phys_data.dyn_friction + b.phys_data.dyn_friction) * 0.5;
        -j * tangent * dyn_friction
    };

    objects.get_mut(&a_idx).unwrap().velocity -= 1. * a.phys_data.inv_mass * friction_impulse;
    objects.get_mut(&b_idx).unwrap().velocity += 1. * b.phys_data.inv_mass * friction_impulse;
}

fn positional_correction(
    objects: &mut Rigidbodies,
    a_idx: Collider_Handle,
    b_idx: Collider_Handle,
    normal: Vec2f,
    penetration: f32,
) {
    trace!("physics::positional_correction");

    let a_inv_mass = objects[&a_idx].phys_data.inv_mass;
    let b_inv_mass = objects[&b_idx].phys_data.inv_mass;

    if a_inv_mass + b_inv_mass == 0. {
        return;
    }

    let correction_perc = 0.2;
    let slop = 0.01;

    let correction =
        (penetration - slop).max(0.0) / (a_inv_mass + b_inv_mass) * correction_perc * normal;

    objects.get_mut(&a_idx).unwrap().position -= a_inv_mass * correction;
    objects.get_mut(&b_idx).unwrap().position += b_inv_mass * correction;
}

fn solve_collisions(objects: &mut Rigidbodies, infos: &[&Collision_Info_Internal]) {
    trace!("physics::solve_collisions");

    for info in infos {
        let Collision_Info_Internal {
            cld1,
            cld2,
            info:
                Collision_Info {
                    normal,
                    penetration,
                    ..
                },
        } = **info;

        solve_collision_velocities(objects, cld1, cld2, normal);
        positional_correction(objects, cld1, cld2, normal, penetration);
    }
}

pub fn update_collisions<T_Spatial_Accelerator>(
    ecs_world: &mut Ecs_World,
    accelerator: &T_Spatial_Accelerator,
    phys_world: &mut Physics_World,
    settings: &Physics_Settings,
    evt_register: &mut Event_Register,
    temp_alloc: &mut Temp_Allocator,
    #[cfg(debug_assertions)] debug_data: &mut Collision_System_Debug_Data,
) where
    T_Spatial_Accelerator: Spatial_Accelerator<Collider_Handle>,
{
    trace!("update_collisions");

    phys_world.clear_collisions();

    update_colliders_spatial(ecs_world, phys_world);

    let infos = detect_collisions(
        phys_world,
        accelerator,
        &settings.collision_matrix,
        temp_alloc,
        #[cfg(debug_assertions)]
        debug_data,
    );

    {
        trace!("add_collisions_to_phys_world");
        infos.iter().for_each(|info| {
            phys_world.add_collision(info.cld1, info.cld2, &info.info);
        });
    }

    let mut objects = prepare_colliders_and_gather_rigidbodies(ecs_world, phys_world);

    let rb_infos = infos
        .par_iter()
        .filter(|info| objects.contains_key(&info.cld1) && objects.contains_key(&info.cld2))
        .collect::<Vec<_>>();

    solve_collisions(&mut objects, &rb_infos);

    // Copy back positions and velocities
    let mut processed = std::collections::HashSet::new();
    for info in &rb_infos {
        let Collision_Info_Internal { cld1, cld2, .. } = info;

        for cld in &[*cld1, *cld2] {
            if !processed.contains(cld) {
                processed.insert(*cld);

                let Rigidbody {
                    position,
                    velocity,
                    entity,
                    offset,
                    ..
                } = objects[cld];

                let mut spatial = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();
                spatial.transform.set_position_v(position - offset);
                spatial.velocity = velocity;
            }
        }
    }

    // Note: we do this last to avoid polluting the cache (we don't know how many observers are
    // subscribed to this event).
    let data: Vec<&Collision_Data> = phys_world
        .collisions
        .values()
        .map(|v| v.as_slice())
        .flatten()
        .collect();
    evt_register.raise_batch::<Evt_Collision_Happened>(&data);
}

fn update_colliders_spatial(ecs_world: &mut Ecs_World, phys_world: &mut Physics_World) {
    trace!("update_colliders_spatial");

    if let Some(mut spatials) = ecs_world.write_component_storage::<C_Spatial2D>() {
        for collider in &mut phys_world.colliders {
            let mut spatial = spatials.must_get_mut(collider.entity);
            let pos = spatial.transform.position();
            spatial.frame_starting_pos = pos;

            collider.position = pos + collider.offset;
            collider.velocity = spatial.velocity;
        }
    }
}

/// Returns { collider => rigidbody }
/// Note that some entities may have non-physical colliders (i.e. trigger colliders),
/// but each entity must have at most 1 physical collider. // :MultipleRigidbodies: lift this restriction!
fn prepare_colliders_and_gather_rigidbodies(
    world: &mut Ecs_World,
    phys_world: &mut Physics_World,
) -> Rigidbodies {
    trace!("prepare_colliders_and_gather_rigidbodies");

    // @Speed: try to use an array rather than a HashMap
    let mut objects = HashMap::new();

    for body in &phys_world.bodies {
        // @Incomplete :MultipleRigidbodies: handle multiple rigidbody colliders
        if let Some(&(cld_handle, phys_data)) = body.rigidbody_colliders.get(0) {
            if let Some(rb_cld) = phys_world.get_collider(cld_handle) {
                objects.insert(
                    cld_handle,
                    Rigidbody {
                        entity: rb_cld.entity,
                        position: rb_cld.position,
                        offset: rb_cld.offset,
                        velocity: rb_cld.velocity,
                        shape: rb_cld.shape,
                        phys_data,
                    },
                );
            }
        }
    }

    objects
}
