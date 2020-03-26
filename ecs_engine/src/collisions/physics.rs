// reference: https://gamedevelopment.tutsplus.com/tutorials/how-to-create-a-custom-2d-physics-engine-the-basics-and-impulse-resolution--gamedev-6331

use crate::common::math::clamp;
use crate::collisions::collider::Collider;
use crate::ecs::components::base::C_Spatial2D;
use crate::collisions::collider::Collision_Shape;
use crate::common::vector::Vec2f;
use crate::ecs::ecs_world::{Entity, Ecs_World};

type Body_Id = u32;

#[derive(Debug, Clone)]
pub struct Rigidbody {
    pub velocity: Vec2f,
    pub position: Vec2f,
    pub shape: Collision_Shape,
    pub inv_mass: f32,
    pub restitution: f32,
    pub entity: Entity,
}

#[derive(Debug, Clone)]
struct Collision_Info {
    pub body1: Body_Id,
    pub body2: Body_Id,
    pub penetration: f32,
    pub normal: Vec2f,
}

fn detect_circle_circle(a_id: Body_Id, b_id: Body_Id, a: &Rigidbody, b: &Rigidbody) -> Option<Collision_Info> {
    let a_radius = if let Collision_Shape::Circle { radius } = a.shape {  radius  } else { panic!("Failed to unwrap Circle!") };
    let b_radius = if let Collision_Shape::Circle { radius } = b.shape {  radius  } else { panic!("Failed to unwrap Circle!") };

    let diff = b.position - a.position;
    let r = a_radius + b_radius;
    let rsquared = r * r;

    if diff.magnitude2() > rsquared {
        return None;
    }

    let dist = diff.magnitude();
    if dist > std::f32::EPSILON {
        Some(Collision_Info {
            body1: a_id,
            body2: b_id,
            normal: diff / dist,
            penetration: r - dist
        })
    } else {
        // circles are in the same position
        Some(Collision_Info {
            body1: a_id,
            body2: b_id,
            normal: v2!(1., 0.), // Arbitrary
            penetration: a_radius, // Arbitrary
        })
    }
}

fn detect_rect_rect(a_id: Body_Id, b_id: Body_Id, a: &Rigidbody, b: &Rigidbody) -> Option<Collision_Info> {
    let (a_width, a_height) = if let Collision_Shape::Rect { width, height } = a.shape {  ( width, height ) } else { panic!("Failed to unwrap Rect!") };
    let (b_width, b_height) = if let Collision_Shape::Rect { width, height } = b.shape { ( width, height ) } else { panic!("Failed to unwrap Rect!") };
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
    if x_overlap > y_overlap {
        let normal = if diff.x < 0. { v2!(-1., 0.) } else { v2!(1., 0.) };
        Some(Collision_Info {
            body1: a_id,
            body2: b_id,
            normal,
            penetration: x_overlap
        })
    } else {
        let normal = if diff.y < 0. { v2!(0., -1.) } else { v2!(0., 1.) };
        Some(Collision_Info {
            body1: a_id,
            body2: b_id,
            normal,
            penetration: y_overlap
        })
    }   
}

fn detect_circle_rect(circle_id: Body_Id, rect_id: Body_Id, circle: &Rigidbody, rect: &Rigidbody) -> Option<Collision_Info> {
    let (r_width, r_height) = if let Collision_Shape::Rect { width, height } = rect.shape {  ( width, height ) } else { panic!("Failed to unwrap Rect!") };
    let c_radius = if let Collision_Shape::Circle { radius } = circle.shape {  radius  } else { panic!("Failed to unwrap Circle!") };

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
        body1: circle_id,
        body2: rect_id,
        normal: if inside { -normal } else { normal },
        penetration: r - d
    })
}

fn detect_collisions(objects: &mut [Rigidbody]) -> Vec<Collision_Info> {
    // TODO Broad phase
    
    // Narrow phase
    let mut collision_infos = vec![];
    let n_objects = objects.len();
    for i in 0..n_objects {
        for j in 0..n_objects {
            if i == j {
                continue;
            }

            let a = &objects[i];
            let b = &objects[j];
            let info = match a.shape {
                Collision_Shape::Circle { .. } => {
                    match b.shape {
                        Collision_Shape::Circle { .. } => {
                            detect_circle_circle(i as _, j as _, a, b)
                        }
                        Collision_Shape::Rect { .. } => {
                            detect_circle_rect(i as _, j as _, a, b)
                        }
                    }
                },
                Collision_Shape::Rect { .. } => {
                    match b.shape {
                        Collision_Shape::Circle { .. } => {
                            detect_circle_rect(j as _, i as _, b, a)
                        }
                        Collision_Shape::Rect { .. } => {
                            detect_rect_rect(i as _, j as _, a, b)
                        }
                    }
                },
            };

            if let Some(info) = info {
                collision_infos.push(info);
            }
        }
    }

    collision_infos
}

// "roughly" because it doesn't do the positional correction
fn solve_collision_roughly(objects: &mut [Rigidbody], a_id: Body_Id, b_id: Body_Id, normal: Vec2f) {
    let a = objects[a_id as usize].clone();
    let b = objects[b_id as usize].clone();

    let rel_vel = b.velocity - a.velocity;
    let vel_along_normal = rel_vel.dot(normal);

    if vel_along_normal > 0. {
        return;
    }

    let e = a.restitution.min(b.restitution);
    
    // Impulse scalar
    let j = -(1. + e) * vel_along_normal / (a.inv_mass + b.inv_mass);

    let impulse = j * normal;
    objects[a_id as usize].velocity -= 1. * a.inv_mass * impulse;
    objects[b_id as usize].velocity += 1. * b.inv_mass * impulse;
}

fn positional_correction(objects: &mut [Rigidbody], a_id: Body_Id, b_id: Body_Id, normal: Vec2f, penetration: f32) {
    let a_inv_mass = objects[a_id as usize].inv_mass;
    let b_inv_mass = objects[b_id as usize].inv_mass;

    let correction_perc = 0.2;
    let slop = 0.01; 

    let correction = (penetration - slop).max(0.0) / (a_inv_mass + b_inv_mass) * correction_perc * normal;

    objects[a_id as usize].position -= a_inv_mass * correction;
    objects[b_id as usize].position += b_inv_mass * correction;
}

fn solve_collisions(objects: &mut[Rigidbody], infos: &[Collision_Info]) {
    for info in infos {
        let Collision_Info {
            body1,
            body2,
            normal,
            penetration
        } = info;

        solve_collision_roughly(objects, *body1, *body2, *normal);
        positional_correction(objects, *body1, *body2, *normal, *penetration);
    }
}

pub fn update_collisions(ecs_world: &mut Ecs_World) {
    // @Temporary @Speed
    for collider in ecs_world.get_components_mut::<Collider>() {
        collider.colliding = false;
    }

    let mut objects = gather_rigidbodies(ecs_world);

    let infos = detect_collisions(&mut objects);

    solve_collisions(&mut objects, &infos);

    // Copy back positions and velocities
    let mut processed = std::collections::HashSet::new();
    for info in &infos {
        let Collision_Info {
            body1,
            body2,
            ..
        } = info;

        for body in &[*body1, *body2] {
            if !processed.contains(body) {
                let Rigidbody {
                    position,
                    velocity,
                    entity,
                    ..
                } = objects[*body as usize];

                let spatial = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();
                // @Incomplete should be global_transform
                spatial.local_transform.set_position_v(position);
                spatial.velocity = velocity;

                ecs_world.get_component_mut::<Collider>(entity).unwrap().colliding = true;
                
                processed.insert(*body);
            }
        }
    }
}

fn gather_rigidbodies(world: &Ecs_World) -> Vec<Rigidbody> {
    use crate::ecs::entity_stream::new_entity_stream;

    let mut objects = vec![]; 

    let mut stream = new_entity_stream(world)
        .require::<Collider>()
        .require::<C_Spatial2D>()
        .build();

    loop {
        let entity = stream.next(world);
        if entity.is_none() {
            break;
        }

        let entity = entity.unwrap();
        let collider = world.get_component::<Collider>(entity).unwrap();
        let spatial = world.get_component::<C_Spatial2D>(entity).unwrap();

        objects.push(Rigidbody {
            position: spatial.global_transform.position(),
            velocity: spatial.velocity,
            shape: collider.shape,
            inv_mass: 1.,
            restitution: 0.1,
            entity,
        });
    }

    objects
}
