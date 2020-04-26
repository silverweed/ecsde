// reference: https://gamedevelopment.tutsplus.com/tutorials/how-to-create-a-custom-2d-physics-engine-the-basics-and-impulse-resolution--gamedev-6331

use crate::collisions::collider::{C_Phys_Data, Collider, Collision_Shape};
use crate::common::math::clamp;
use crate::common::vector::{sanity_check_v, Vec2f};
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::ecs_world::{Component_Storage, Ecs_World, Entity};
use rayon::prelude::*;
use std::collections::HashMap;

type Body_Id = u32;

#[derive(Debug, Clone)]
struct Rigidbody {
    // Used in detect_collisions
    pub shape: Collision_Shape,

    // Used to copy back results
    pub entity: Entity,

    // Used in positional_correction
    pub position: Vec2f, // and copy back // and detect collision

    // Used in solve_collisions_roughly
    pub velocity: Vec2f, // and copy back
    pub phys_data: C_Phys_Data,
}

#[derive(Debug, Clone)]
struct Collision_Info {
    pub body1: Entity,
    pub body2: Entity,
    pub penetration: f32,
    pub normal: Vec2f,
}

fn detect_circle_circle(
    ent_a: Entity,
    ent_b: Entity,
    a: &Collider,
    b: &Collider,
) -> Option<Collision_Info> {
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
            body1: ent_a,
            body2: ent_b,
            normal: diff / dist,
            penetration: r - dist,
        })
    } else {
        // circles are in the same position
        Some(Collision_Info {
            body1: ent_a,
            body2: ent_b,
            normal: v2!(1., 0.),   // Arbitrary
            penetration: a_radius, // Arbitrary
        })
    }
}

fn detect_rect_rect(
    ent_a: Entity,
    ent_b: Entity,
    a: &Collider,
    b: &Collider,
) -> Option<Collision_Info> {
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
            body1: ent_a,
            body2: ent_b,
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
            body1: ent_a,
            body2: ent_b,
            normal,
            penetration: y_overlap,
        })
    }
}

#[allow(clippy::collapsible_if)]
fn detect_circle_rect(
    ent_circle: Entity,
    ent_rect: Entity,
    circle: &Collider,
    rect: &Collider,
) -> Option<Collision_Info> {
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
        body1: ent_circle,
        body2: ent_rect,
        normal: if inside { -normal } else { normal },
        penetration: r - d,
    })
}

fn detect_rect_circle(
    ent_rect: Entity,
    ent_circle: Entity,
    rect: &Collider,
    circle: &Collider,
) -> Option<Collision_Info> {
    detect_circle_rect(ent_circle, ent_rect, circle, rect)
}

fn collision_shape_type_index(shape: &Collision_Shape) -> usize {
    match shape {
        Collision_Shape::Circle { .. } => 0,
        Collision_Shape::Rect { .. } => 1,
    }
}

type Collision_Cb = fn(Entity, Entity, &Collider, &Collider) -> Option<Collision_Info>;

const COLLISION_CB_TABLE: [[Collision_Cb; 2]; 2] = [
    [detect_circle_circle, detect_circle_rect],
    [detect_rect_circle, detect_rect_rect],
];

fn detect_collisions(
    colliders: &Component_Storage<'_, Collider>,
    entities: &[Entity],
) -> Vec<Collision_Info> {
    trace!("physics::detect_collisions");

    // TODO Broad phase

    // Narrow phase
    let mut collision_infos = vec![];
    // @Speed
    let mut stored = std::collections::HashSet::new();
    for (i, &ent_a) in entities.iter().enumerate() {
        let a = colliders.get_component(ent_a).unwrap();
        if a.is_static {
            continue;
        }
        let a_shape = collision_shape_type_index(&a.shape);
        let a_part_cb = COLLISION_CB_TABLE[a_shape];

        for (j, &ent_b) in entities.iter().enumerate() {
            if i == j {
                continue;
            }
            let b = colliders.get_component(ent_b).unwrap();
            let b_shape = collision_shape_type_index(&b.shape);

            let info = a_part_cb[b_shape](ent_a, ent_b, a, b);

            if let Some(info) = info {
                if !stored.contains(&(j, i)) {
                    collision_infos.push(info);
                    stored.insert((i, j));
                }
            }
        }
    }

    collision_infos
}

// "roughly" because it doesn't do the positional correction
fn solve_collision_roughly(
    objects: &mut HashMap<Entity, Rigidbody>,
    a_id: Entity,
    b_id: Entity,
    normal: Vec2f,
) {
    trace!("physics::solve_collisions_roughly");

    let a = objects[&a_id].clone();
    let b = objects[&b_id].clone();

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
    objects.get_mut(&a_id).unwrap().velocity -= 1. * a.phys_data.inv_mass * impulse;
    objects.get_mut(&b_id).unwrap().velocity += 1. * b.phys_data.inv_mass * impulse;

    // @Speed!
    let a = objects[&a_id].clone();
    let b = objects[&b_id].clone();

    // apply friction
    let new_rel_vel = b.velocity - a.velocity;
    sanity_check_v(a.velocity);
    sanity_check_v(b.velocity);
    sanity_check_v(new_rel_vel);

    let tangent = (new_rel_vel - new_rel_vel.dot(normal) * normal).normalized_or_zero();

    let jt = -new_rel_vel.dot(tangent) / (a.phys_data.inv_mass * b.phys_data.inv_mass);

    fn pythag(a: f32, b: f32) -> f32 {
        (a * a + b * b).sqrt()
    }

    // @Speed: try to use another method here (e.g. average)
    let mu = pythag(a.phys_data.static_friction, b.phys_data.static_friction);

    let friction_impulse = if jt.abs() < j * mu {
        jt * tangent
    } else {
        let dyn_friction = pythag(a.phys_data.dyn_friction, b.phys_data.dyn_friction);
        -j * tangent * dyn_friction
    };

    objects.get_mut(&a_id).unwrap().velocity -= 1. * a.phys_data.inv_mass * friction_impulse;
    objects.get_mut(&b_id).unwrap().velocity += 1. * b.phys_data.inv_mass * friction_impulse;
}

fn positional_correction(
    objects: &mut HashMap<Entity, Rigidbody>,
    a_id: Entity,
    b_id: Entity,
    normal: Vec2f,
    penetration: f32,
) {
    trace!("physics::positional_correction");

    let a_inv_mass = objects[&a_id].phys_data.inv_mass;
    let b_inv_mass = objects[&b_id].phys_data.inv_mass;

    if a_inv_mass + b_inv_mass == 0. {
        return;
    }

    let correction_perc = 0.2;
    let slop = 0.01;

    let correction =
        (penetration - slop).max(0.0) / (a_inv_mass + b_inv_mass) * correction_perc * normal;

    objects.get_mut(&a_id).unwrap().position -= a_inv_mass * correction;
    objects.get_mut(&b_id).unwrap().position += b_inv_mass * correction;
}

fn solve_collisions(objects: &mut HashMap<Entity, Rigidbody>, infos: &[&Collision_Info]) {
    trace!("physics::solve_collisions");

    for info in infos {
        let Collision_Info {
            body1,
            body2,
            normal,
            penetration,
        } = **info;

        solve_collision_roughly(objects, body1, body2, normal);
        positional_correction(objects, body1, body2, normal, penetration);
    }
}

pub fn update_collisions(ecs_world: &mut Ecs_World) {
    let (mut objects, entities) = prepare_colliders_and_gather_rigidbodies(ecs_world);

    let colliders = ecs_world.get_component_storage::<Collider>();

    let infos = detect_collisions(&colliders, &entities);

    let mut colliders = ecs_world.get_component_storage_mut::<Collider>();

    infos.iter().for_each(|info| {
        colliders
            .get_component_mut(info.body1)
            .unwrap()
            .colliding_with = Some(info.body2);
        colliders
            .get_component_mut(info.body2)
            .unwrap()
            .colliding_with = Some(info.body1);
    });

    let rb_infos = infos
        .par_iter()
        .filter(|info| objects.contains_key(&info.body1) && objects.contains_key(&info.body2))
        .collect::<Vec<_>>();
    solve_collisions(&mut objects, &rb_infos);

    // Copy back positions and velocities
    let mut processed = std::collections::HashSet::new();
    for info in &rb_infos {
        let Collision_Info { body1, body2, .. } = info;

        for body in &[*body1, *body2] {
            if !processed.contains(body) {
                processed.insert(*body);

                let Rigidbody {
                    position,
                    velocity,
                    entity,
                    ..
                } = objects[body];

                let spatial = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();
                spatial.transform.set_position_v(position);
                spatial.velocity = velocity;
            }
        }
    }
}

fn prepare_colliders_and_gather_rigidbodies(
    world: &mut Ecs_World,
) -> (HashMap<Entity, Rigidbody>, Vec<Entity>) {
    // @Speed: try to use an array rather than a HashMap
    let mut objects = HashMap::new();
    let mut entities = vec![];

    foreach_entity!(world, +Collider, +C_Spatial2D, |entity| {
        let spatial = world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        let pos = spatial.transform.position();
        let velocity = spatial.velocity;
        sanity_check_v(velocity);
        spatial.frame_starting_pos = pos;

        let collider = world.get_component_mut::<Collider>(entity).unwrap();
        collider.position = pos;
        collider.colliding_with = None;
        let position = collider.position;
        let shape = collider.shape;

        entities.push(entity);

        if let Some(phys_data) = world.get_component::<C_Phys_Data>(entity) {
            objects.insert(entity, Rigidbody {
                entity,
                position,
                velocity,
                shape,
                phys_data: *phys_data,
            });
        }
    });

    (objects, entities)
}
