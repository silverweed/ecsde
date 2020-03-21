use crate::systems::controllable_system::C_Controllable;
use ecs_engine::collisions::collider::Collider;
use ecs_engine::common::angle::deg;
use ecs_engine::core::rand::{self, Default_Rng};
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::{ecs_world::Ecs_World, entity_stream};
use std::time::Duration;

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Dumb_Movement {
    time_since_change: Duration,
}

const MIN_T_TO_CHANGE: Duration = Duration::from_millis(500);

pub fn update(dt: &Duration, ecs_world: &mut Ecs_World, rng: &mut Default_Rng) {
    let mut entity_stream = entity_stream::new_entity_stream(ecs_world)
        .require::<C_Spatial2D>()
        .require::<C_Dumb_Movement>()
        .require::<Collider>()
        .exclude::<C_Controllable>()
        .build();
    loop {
        let entity = entity_stream.next(ecs_world);
        if entity.is_none() {
            break;
        }
        let entity = entity.unwrap();

        let dumb_movement = ecs_world
            .get_component_mut::<C_Dumb_Movement>(entity)
            .unwrap();
        dumb_movement.time_since_change += *dt;
        if dumb_movement.time_since_change < MIN_T_TO_CHANGE {
            continue;
        }

        let collider = ecs_world.get_component::<Collider>(entity).unwrap();
        let colliding = collider.colliding;

        let spatial = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        if spatial.velocity.magnitude2() < 0.1 {
            spatial.velocity = v2!(0., -200.);
        }
        if !colliding {
            continue;
        }

        let spatial = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        let r = rand::rand_range(rng, -50., 50.);
        spatial.velocity = -spatial.velocity.rotated(deg(r));

        let dumb_movement = ecs_world
            .get_component_mut::<C_Dumb_Movement>(entity)
            .unwrap();
        dumb_movement.time_since_change = Duration::default();
    }
}
