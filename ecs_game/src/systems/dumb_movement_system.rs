use crate::systems::controllable_system::C_Controllable;
use ecs_engine::collisions::collider::Collider;
use ecs_engine::common::angle::deg;
use ecs_engine::core::rand::{self, Default_Rng};
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::ecs_world::Ecs_World;
use std::time::Duration;

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Dumb_Movement {
    time_since_change: Duration,
}

const MIN_T_TO_CHANGE: Duration = Duration::from_millis(500);

pub fn update(dt: &Duration, ecs_world: &mut Ecs_World, rng: &mut Default_Rng) {
    foreach_entity!(ecs_world, +C_Spatial2D, +C_Dumb_Movement, +Collider, ~C_Controllable, |entity| {
        let dumb_movement = ecs_world
            .get_component_mut::<C_Dumb_Movement>(entity)
            .unwrap();
        dumb_movement.time_since_change += *dt;
        //if dumb_movement.time_since_change < MIN_T_TO_CHANGE {
        //    return;
        //}

        let collider = ecs_world.get_component::<Collider>(entity).unwrap();
        let colliding_with = collider.colliding_with;

        let spatial = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        if spatial.velocity.magnitude2() < 0.1 {
            spatial.velocity = v2!(0., 200.);
        }
        if colliding_with.is_none() {
            return;
        }

        let spatial = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        //let r = rand::rand_range(rng, -50., 50.);
        let r = rand::rand_range(rng, 0., 360.);
        spatial.velocity = - 200. * v2!(1., 0.).rotated(deg(r));

        let dumb_movement = ecs_world
            .get_component_mut::<C_Dumb_Movement>(entity)
            .unwrap();
        dumb_movement.time_since_change = Duration::default();

        if rand::rand_01(rng) < 1.2 {
            let to_destroy = colliding_with.unwrap();
            if ecs_world.is_valid_entity(to_destroy) {
                if let Some(cld) = ecs_world.get_component::<Collider>(to_destroy) {
                    if cld.is_static {
                        ecs_world.destroy_entity(to_destroy);
                    }
                }
            }
        }
    });
}
