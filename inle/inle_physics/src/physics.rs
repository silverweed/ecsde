// reference: https://gamedevelopment.tutsplus.com/tutorials/how-to-create-a-custom-2d-physics-engine-the-basics-and-impulse-resolution--gamedev-6331

use super::layers::Collision_Matrix;
use super::phys_world::{Collider_Handle, Collision_Data, Collision_Info, Physics_World};
use super::spatial::Spatial_Accelerator;
use crate::collider::{Collider, Collision_Shape, Phys_Data};
use inle_alloc::temp::{excl_temp_array, Temp_Allocator};
use inle_events::evt_register::{Event, Event_Register};
use inle_math::math::clamp;
use inle_math::vector::{sanity_check_v, Vec2f};
use std::collections::{HashMap, HashSet};

pub struct Evt_Collision_Happened;

impl Event for Evt_Collision_Happened {
    type Args = Collision_Data;
}

#[derive(Default)]
pub struct Physics_Settings {
    pub collision_matrix: Collision_Matrix,
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

    // TODO: handle non-AA rects

    const OVERLAP_EPSILON: f32 = f32::EPSILON;

    let Collision_Shape::Rect {
        width: a_width,
        height: a_height,
    } = a.shape
    else {
        panic!("Failed to unwrap Rect!")
    };
    let Collision_Shape::Rect {
        width: b_width,
        height: b_height,
    } = b.shape
    else {
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
            // a is on the right, normal points to the left (towards b)
            v2!(-1., 0.)
        } else {
            // a is on the left, normal points to the right (towards b)
            v2!(1., 0.)
        };
        debug_assert!(x_overlap > 0.);
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
            // a is below b, normal points upwards (towards b)
            v2!(0., -1.)
        } else {
            // a is above b, normal points downwards (towards b)
            v2!(0., 1.)
        };
        debug_assert!(y_overlap > 0.);
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

    let Collision_Shape::Rect {
        width: r_width,
        height: r_height,
    } = rect.shape
    else {
        panic!("Failed to unwrap Rect!")
    };
    let Collision_Shape::Circle { radius: c_radius } = circle.shape else {
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
        trace!("iterate_colliders");

        let a_extent = a.shape.extent();
        let a_shape = collision_shape_type_index(&a.shape);
        let a_partial_cb = COLLISION_CB_TABLE[a_shape];

        let mut neighbours = excl_temp_array(temp_alloc);
        accelerator.get_neighbours(a.position, a_extent, phys_world, &mut neighbours);

        for &b_handle in &neighbours {
            if a.handle == b_handle {
                continue;
            }
            let b = phys_world.get_collider(b_handle).unwrap();
            let b_shape = collision_shape_type_index(&b.shape);

            let pa: *const Collider = a as *const _;
            let pb: *const Collider = b as *const _;

            if !collision_matrix.layers_collide(a.layer, b.layer) || storage.contains(&(pb, pa)) {
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

fn solve_collision_velocities(a: &mut Collider, b: &mut Collider, normal: Vec2f) {
    trace!("physics::solve_collisions_velocities");

    let a_phys = a.phys_data.unwrap();
    let b_phys = b.phys_data.unwrap();
    if a_phys.inv_mass + b_phys.inv_mass == 0. {
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

    let e = a_phys.restitution.min(b_phys.restitution);

    // Impulse scalar
    let j = -(1. + e) * vel_along_normal / (a_phys.inv_mass + b_phys.inv_mass);
    debug_assert!(!j.is_nan());

    let impulse = j * normal;
    a.velocity -= 1. * a_phys.inv_mass * impulse;
    a.velocity += 1. * b_phys.inv_mass * impulse;

    // apply friction
    let new_rel_vel = b.velocity - a.velocity;
    sanity_check_v(a.velocity);
    sanity_check_v(b.velocity);
    sanity_check_v(new_rel_vel);

    let tangent = (new_rel_vel - new_rel_vel.dot(normal) * normal).normalized_or_zero();

    let jt = -new_rel_vel.dot(tangent) / (a_phys.inv_mass * b_phys.inv_mass);

    let mu = (a_phys.static_friction + b_phys.static_friction) * 0.5;

    let friction_impulse = if jt.abs() < j * mu {
        jt * tangent
    } else {
        let dyn_friction = (a_phys.dyn_friction + b_phys.dyn_friction) * 0.5;
        -j * tangent * dyn_friction
    };

    a.velocity -= 1. * a_phys.inv_mass * friction_impulse;
    b.velocity += 1. * b_phys.inv_mass * friction_impulse;
}

fn positional_correction(a: &mut Collider, b: &mut Collider, normal: Vec2f, penetration: f32) {
    trace!("physics::positional_correction");

    let a_inv_mass = a.phys_data.unwrap().inv_mass;
    let b_inv_mass = b.phys_data.unwrap().inv_mass;

    if a_inv_mass + b_inv_mass == 0. {
        // Both infinite-mass objects.
        return;
    }

    let correction_perc = 0.2;
    let slop = 0.01;

    let correction =
        (penetration - slop).max(0.0) / (a_inv_mass + b_inv_mass) * correction_perc * normal;

    a.position -= a_inv_mass * correction;
    b.position += b_inv_mass * correction;
}

fn solve_collisions(phys_world: &mut Physics_World, infos: &[Collision_Info_Internal]) {
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
        } = *info;

        let (cld1, cld2) = phys_world.get_collider_pair_mut(cld1, cld2).unwrap();
        if cld1.phys_data.is_some() && cld2.phys_data.is_some() {
            solve_collision_velocities(cld1, cld2, normal);
            positional_correction(cld1, cld2, normal, penetration);
        }
    }
}

pub fn update_collisions<T_Spatial_Accelerator>(
    accelerator: &T_Spatial_Accelerator,
    phys_world: &mut Physics_World,
    settings: &Physics_Settings,
    evt_register: Option<&mut Event_Register>,
    temp_alloc: &mut Temp_Allocator,
    #[cfg(debug_assertions)] debug_data: &mut Collision_System_Debug_Data,
) where
    T_Spatial_Accelerator: Spatial_Accelerator<Collider_Handle>,
{
    trace!("update_collisions");

    phys_world.clear_collisions();

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

    solve_collisions(phys_world, &infos);

    // Note: we do this last to avoid polluting the cache (we don't know how many observers are
    // subscribed to this event).
    if let Some(evt_register) = evt_register {
        let data: Vec<&Collision_Data> = phys_world
            .collisions
            .values()
            .flat_map(|v| v.as_slice())
            .collect();
        evt_register.raise_batch::<Evt_Collision_Happened>(&data);
    }
}
