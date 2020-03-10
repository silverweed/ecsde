use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::ecs_world::Ecs_World;
use std::time::Duration;

pub fn update(dt: &Duration, ecs_world: &mut Ecs_World) {
    let dt_secs = dt.as_secs_f32();

    for spatial in ecs_world.get_components_mut::<C_Spatial2D>() {
        let translation = spatial.velocity * dt_secs;
        spatial.local_transform.translate_v(translation);
    }
}
