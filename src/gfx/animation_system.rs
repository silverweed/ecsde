use crate::core::time;
use crate::ecs::components::C_Renderable;
use crate::ecs::entity_manager::Entity_Manager;
use std::time::Duration;

pub fn update(dt: &Duration, em: &mut Entity_Manager) {
    let dt_secs = time::to_secs_frac(&dt);
    let mut animated_sprites = em.get_components_mut::<C_Renderable>();

    for sprite in animated_sprites
        .iter_mut()
        .filter(|sprite| sprite.frame_time > 0.0 && sprite.n_frames > 1)
    {
        sprite.frame_time_elapsed += dt_secs;

        if sprite.frame_time_elapsed >= sprite.frame_time {
            sprite.frame_time_elapsed = 0.0;

            let width = sprite.rect.width();
            let x = (sprite.rect.x + width as i32) % (width * (sprite.n_frames - 1)) as i32;

            sprite.rect.x = x;
        }
    }
}
