use ecs_engine::core::time;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::ecs_world::Ecs_World;
use std::time::Duration;

// @Fixme: something is going wrong with this + smooth_by_interpolating_velocity: investigate
pub fn update(dt: &Duration, ecs_world: &mut Ecs_World) {
    let dt_secs = time::to_secs_frac(&dt);

    for spatial in ecs_world.get_components_mut::<C_Spatial2D>() {
        let translation = spatial.velocity * dt_secs;
        spatial.local_transform.translate_v(translation);
    }
}
