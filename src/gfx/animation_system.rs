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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::env::Env_Info;
    use crate::gfx;
    use crate::resources::{self, tex_path, Resources};
    use crate::test_common;
    use sdl2::rect::Rect;

    #[test]
    fn animation_system() {
        let (mut rsrc, env) = test_common::create_test_resources_and_env();
        let mut em = Entity_Manager::new();
        em.register_component::<C_Renderable>();

        let e = em.new_entity();
        {
            let mut r = em.add_component::<C_Renderable>(e);
            r.texture = rsrc.load_texture(&tex_path(&env, "plant.png"));
            r.rect = Rect::new(0, 0, 96, 96);
            r.n_frames = 4;
            r.frame_time = 0.1;
        }

        let mut dt = Duration::from_millis(16);
        for i in 0..1000 {
            update(&dt, &mut em);
            let r = em.get_component::<C_Renderable>(e).unwrap();
            assert!(
                r.rect.x % r.rect.width() as i32 == 0,
                "sprite from spritesheet is not aligned! (x = {} not multiple of sprite width {})",
                r.rect.x,
                r.rect.width()
            );

            if i % 50 == 0 && i > 0 {
                dt = Duration::from_millis((i / 50) * (i / 50) * 2);
            }
        }
    }
}
