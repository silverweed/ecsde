// reference: https://gamedevelopment.tutsplus.com/tutorials/how-to-create-a-custom-2d-physics-engine-the-basics-and-impulse-resolution--gamedev-6331

use super::layers::Collision_Matrix;
use super::phys_world::{Collider_Handle, Phys_Data, Physics_World};
use super::spatial::Spatial_Accelerator;
use crate::collider::{Collider, Collision_Shape};
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::{Ecs_World, Entity};
use inle_events::evt_register::{Event, Event_Register};
use inle_math::math::clamp;
use inle_math::vector::{sanity_check_v, Vec2f};
use rayon::prelude::*;
use std::collections::HashMap;

type Rigidbodies = HashMap<usize, Rigidbody>;

pub struct Evt_Collision_Happened;

impl Event for Evt_Collision_Happened {
    type Args = (Collider, Collider);
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
struct Collision_Info {
    // Note: cld1 and cld2 contain pointers to colliders. We're saving them as usize
    // so this struct remains thread-safe (we're not doing pointery stuff from other
    // threads: just using them as hashmap keys, so it's safe).
    pub cld1: usize,
    pub cld2: usize,
    pub penetration: f32,
    pub normal: Vec2f,
}

#[cfg(debug_assertions)]
#[derive(Default)]
pub struct Collision_System_Debug_Data {
    // How many intersections were tested during this frame
    pub n_intersection_tests: usize,
}

fn detect_circle_circle(a: &Collider, b: &Collider) -> Option<Collision_Info> {
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
        Some(Collision_Info {
            cld1: a as *const _ as usize,
            cld2: b as *const _ as usize,
            normal: diff / dist,
            penetration: r - dist,
        })
    } else {
        // circles are in the same position
        Some(Collision_Info {
            cld1: a as *const _ as usize,
            cld2: b as *const _ as usize,
            normal: v2!(1., 0.),   // Arbitrary
            penetration: a_radius, // Arbitrary
        })
    }
}

fn detect_rect_rect(a: &Collider, b: &Collider) -> Option<Collision_Info> {
    trace!("physics::detect_rect_rect");

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
    if x_overlap <= 0. {
        return None;
    }

    let a_half_ext_y = a_height * 0.5;
    let b_half_ext_y = b_height * 0.5;

    // Apply SAT on Y axis
    let y_overlap = a_half_ext_y + b_half_ext_y - diff.y.abs();
    if y_overlap <= 0. {
        return None;
    }

    // Find least penetration axis
    if x_overlap < y_overlap {
        let normal = if diff.x < 0. {
            v2!(-1., 0.)
        } else {
            v2!(1., 0.)
        };
        Some(Collision_Info {
            cld1: a as *const _ as usize,
            cld2: b as *const _ as usize,
            normal,
            penetration: x_overlap,
        })
    } else {
        let normal = if diff.y < 0. {
            v2!(0., -1.)
        } else {
            v2!(0., 1.)
        };
        Some(Collision_Info {
            cld1: a as *const _ as usize,
            cld2: b as *const _ as usize,
            normal,
            penetration: y_overlap,
        })
    }
}

#[allow(clippy::collapsible_if, clippy::collapsible_else_if)]
fn detect_circle_rect(circle: &Collider, rect: &Collider) -> Option<Collision_Info> {
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

    Some(Collision_Info {
        cld1: circle as *const _ as usize,
        cld2: rect as *const _ as usize,
        normal: if inside { -normal } else { normal },
        penetration: r - d,
    })
}

fn detect_rect_circle(rect: &Collider, circle: &Collider) -> Option<Collision_Info> {
    detect_circle_rect(circle, rect)
}

fn collision_shape_type_index(shape: &Collision_Shape) -> usize {
    match shape {
        Collision_Shape::Circle { .. } => 0,
        Collision_Shape::Rect { .. } => 1,
    }
}

type Collision_Cb = fn(&Collider, &Collider) -> Option<Collision_Info>;

const COLLISION_CB_TABLE: [[Collision_Cb; 2]; 2] = [
    [detect_circle_circle, detect_circle_rect],
    [detect_rect_circle, detect_rect_rect],
];

fn detect_collisions<T_Spatial_Accelerator>(
    phys_world: &Physics_World,
    accelerator: &T_Spatial_Accelerator,
    collision_matrix: &Collision_Matrix,
    #[cfg(debug_assertions)] debug_data: &mut Collision_System_Debug_Data,
) -> Vec<Collision_Info>
where
    T_Spatial_Accelerator: Spatial_Accelerator<Collider_Handle>,
{
    trace!("physics::detect_collisions");

    #[cfg(debug_assertions)]
    {
        debug_data.n_intersection_tests = 0;
    }

    let mut collision_infos = vec![];
    let mut stored = std::collections::HashSet::new();

    // @Speed: maybe we should iterate on the chunks? Can we do that in parallel?
    for a in phys_world.colliders.iter() {
        if a.is_static {
            continue;
        }
        let a_extent = a.shape.extent();
        let a_shape = collision_shape_type_index(&a.shape);
        let a_part_cb = COLLISION_CB_TABLE[a_shape];
        let ent_a = a.entity;

        let mut neighbours = vec![];
        accelerator.get_neighbours(a.position, a_extent, &mut neighbours);

        for b_handle in neighbours {
            let b = phys_world.get_collider(b_handle).unwrap();
            let ent_b = b.entity;
            if ent_a == ent_b {
                continue;
            }
            let b_shape = collision_shape_type_index(&b.shape);
            if !collision_matrix.layers_collide(a.layer, b.layer)
                || stored.contains(&(&b as *const _, &a as *const _))
            {
                continue;
            }

            let info = a_part_cb[b_shape](a, b);

            #[cfg(debug_assertions)]
            {
                debug_data.n_intersection_tests += 1;
            }

            if let Some(info) = info {
                collision_infos.push(info);
                stored.insert((&a as *const _, &b as *const _));
            }
        }
    }

    collision_infos
}

// "roughly" because it doesn't do the positional correction
fn solve_collision_roughly(objects: &mut Rigidbodies, a_idx: usize, b_idx: usize, normal: Vec2f) {
    trace!("physics::solve_collisions_roughly");

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
    a_idx: usize,
    b_idx: usize,
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

fn solve_collisions(objects: &mut Rigidbodies, infos: &[&Collision_Info]) {
    trace!("physics::solve_collisions");

    for info in infos {
        let Collision_Info {
            cld1,
            cld2,
            normal,
            penetration,
        } = **info;

        solve_collision_roughly(objects, cld1, cld2, normal);
        positional_correction(objects, cld1, cld2, normal, penetration);
    }
}

pub fn update_collisions<T_Spatial_Accelerator>(
    ecs_world: &mut Ecs_World,
    accelerator: &T_Spatial_Accelerator,
    phys_world: &mut Physics_World,
    settings: &Physics_Settings,
    evt_register: &mut Event_Register,
    #[cfg(debug_assertions)] debug_data: &mut Collision_System_Debug_Data,
) where
    T_Spatial_Accelerator: Spatial_Accelerator<Collider_Handle>,
{
    let mut objects = prepare_colliders_and_gather_rigidbodies(ecs_world, phys_world);

    let infos = detect_collisions(
        phys_world,
        accelerator,
        &settings.collision_matrix,
        #[cfg(debug_assertions)]
        debug_data,
    );

    // @Audit: this is super unsafe! Transmuting `usizes` back to pointers.
    // This should work fine, because Colliders should not be moved during the
    // collision update, so the addresses should stay valid. The scariest point
    // is whether it is UB or not converting to mut ptrs, but, I mean, why should it, right??
    //
    // !!! BY THE WAY !!!
    // Probably we don't want to parallelize this ever. That would just throw more
    // potential UB to us.
    infos.iter().for_each(|info| unsafe {
        let body2 = (*(info.cld2 as *const Collider)).entity;
        (*(info.cld1 as *mut Collider)).colliding_with.push(body2);
        let body1 = (*(info.cld1 as *const Collider)).entity;
        (*(info.cld2 as *mut Collider)).colliding_with.push(body1);

        evt_register.raise::<Evt_Collision_Happened>((
            (*(info.cld1 as *const Collider)).clone(),
            (*(info.cld2 as *const Collider)).clone(),
        ));
    });

    // Note: this can be parallel because we're not actually using pointers; they're just
    // innocent usizes used as keys.
    let rb_infos = infos
        .par_iter()
        .filter(|info| objects.contains_key(&info.cld1) && objects.contains_key(&info.cld2))
        .collect::<Vec<_>>();

    solve_collisions(&mut objects, &rb_infos);

    // Copy back positions and velocities
    let mut processed = std::collections::HashSet::new();
    for info in &rb_infos {
        let Collision_Info { cld1, cld2, .. } = info;

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

                let spatial = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();
                spatial.transform.set_position_v(position - offset);
                spatial.velocity = velocity;
            }
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
    // @Speed: try to use an array rather than a HashMap
    let mut objects = HashMap::new();

    for collider in &mut phys_world.colliders {
        let spatial = world
            .get_component_mut::<C_Spatial2D>(collider.entity)
            .unwrap();
        let pos = spatial.transform.position();
        spatial.frame_starting_pos = pos;

        collider.position = pos + collider.offset;
        collider.colliding_with.clear();
    }

    for body in &phys_world.bodies {
        // @Incomplete :MultipleRigidbodies: handle multiple rigidbody colliders
        if let Some(&(cld_handle, phys_data)) = body.rigidbody_colliders.get(0) {
            if let Some(rb_cld) = phys_world.get_collider(cld_handle) {
                let spatial = world.get_component::<C_Spatial2D>(rb_cld.entity).unwrap();
                let velocity = spatial.velocity;
                sanity_check_v(velocity);

                objects.insert(
                    // @Audit: converting the memory address to a usize to use as an identifier.
                    // If this turns out to be a bad idea, consider putting an ID inside the Collider struct.
                    rb_cld as *const _ as usize,
                    Rigidbody {
                        entity: rb_cld.entity,
                        position: rb_cld.position,
                        offset: rb_cld.offset,
                        velocity,
                        shape: rb_cld.shape,
                        phys_data,
                    },
                );
            }
        }
    }

    objects
}
