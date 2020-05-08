use ecs_engine::alloc::temp::*;
use ecs_engine::collisions::collider::Collider;
use ecs_engine::common::vector::Vec2f;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::ecs_world::{Ecs_World, Entity};
use std::time::Duration;

#[derive(Copy, Clone)]
pub struct Moved_Entity {
    pub entity: Entity,
    pub prev_pos: Vec2f,
    pub new_pos: Vec2f,
    pub extent: Vec2f,
}

const MIN_SPEED: f32 = 0.001;

pub fn update(
    dt: &Duration,
    ecs_world: &mut Ecs_World,
    moved: &mut Exclusive_Temp_Array<Moved_Entity>,
) {
    let dt_secs = dt.as_secs_f32();

    foreach_entity!(ecs_world, +C_Spatial2D, |entity| {
        let spatial = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        if spatial.velocity.magnitude() < MIN_SPEED {
            spatial.velocity = v2!(0., 0.);
        }
        let translation = spatial.velocity * dt_secs;
        spatial.transform.translate_v(translation);

        let pos = spatial.transform.position();
        let starting_pos = spatial.frame_starting_pos;
        if (pos - starting_pos).magnitude2() > std::f32::EPSILON {
            if let Some(collider) = ecs_world.get_component::<Collider>(entity) {
                moved.push(Moved_Entity {
                    entity,
                    prev_pos: starting_pos,
                    new_pos: pos,
                    extent: collider.shape.extent(),
                });
            }
        }
    });
}
