use ecs_engine::common::vector::Vec2f;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::ecs_world::{Ecs_World, Entity};
use std::time::Duration;

pub struct Moved_Entity {
    pub entity: Entity,
    pub prev_pos: Vec2f,
    pub new_pos: Vec2f,
}

pub fn update(dt: &Duration, ecs_world: &mut Ecs_World, moved: &mut Vec<Moved_Entity>) {
    let dt_secs = dt.as_secs_f32();

    foreach_entity!(ecs_world, +C_Spatial2D, |entity| {
        let spatial = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        let translation = spatial.velocity * dt_secs;
        spatial.transform.translate_v(translation);

        if (spatial.transform.position() - spatial.frame_starting_pos).magnitude2() > std::f32::EPSILON {
            moved.push(Moved_Entity {
                entity,
                prev_pos: spatial.frame_starting_pos,
                new_pos: spatial.transform.position(),
            });
        }
    });
}
