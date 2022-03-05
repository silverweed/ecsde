use super::interface::Game_System;
use inle_alloc::temp::*;
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_query_new::Ecs_Query;
use inle_ecs::ecs_world::Ecs_World;
use inle_math::vector::Vec2f;
use inle_physics::collider::C_Collider;
use inle_physics::phys_world::{Collider_Handle, Physics_World};
use std::time::Duration;

#[derive(Copy, Clone)]
pub struct Moved_Collider {
    pub handle: Collider_Handle,
    pub prev_pos: Vec2f,
    pub new_pos: Vec2f,
    pub extent: Vec2f,
}

const MIN_SPEED: f32 = 3.0;

pub struct Movement_System {
    query: Ecs_Query,
}

impl Movement_System {
    pub fn new() -> Self {
        Self {
            query: Ecs_Query::default().require::<C_Spatial2D>(),
        }
    }

    pub fn update_physics(
        &self,
        dt: &Duration,
        ecs_world: &mut Ecs_World,
        phys_world: &Physics_World,
        moved: &mut Exclusive_Temp_Array<Moved_Collider>,
    ) {
        trace!("movement_system::update_physics");

        let dt_secs = dt.as_secs_f32();

        foreach_entity!(self.query, ecs_world,
            read: ;
            write: C_Spatial2D;
        |entity, (), (spatial,): (&mut C_Spatial2D,)| {
            if spatial.velocity.magnitude2() < MIN_SPEED * MIN_SPEED {
                spatial.velocity = v2!(0., 0.);
            }

            let translation = spatial.velocity * dt_secs;
            spatial.transform.translate_v(translation);

            let pos = spatial.transform.position();
            let starting_pos = spatial.frame_starting_pos;
            if (pos - starting_pos).magnitude2() > std::f32::EPSILON {
                if let Some(collider) = ecs_world.get_component::<C_Collider>(entity) {
                    for (collider, handle) in phys_world.
                            get_all_colliders_with_handles(collider.phys_body_handle)
                            .filter(|(cld, _)| !cld.is_static) {
                        moved.push(Moved_Collider {
                            handle,
                            prev_pos: starting_pos + collider.offset,
                            new_pos: pos + collider.offset,
                            extent: collider.shape.extent(),
                        });
                    }
                }
            }
        });
    }
}

impl Game_System for Movement_System {
    fn get_queries_mut(&mut self) -> Vec<&mut Ecs_Query> {
        vec![&mut self.query]
    }
}
