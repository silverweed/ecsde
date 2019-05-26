use crate::core::time;
use crate::ecs::components::gfx::{C_Animated_Sprite, C_Renderable};
use crate::ecs::entity_manager::Entity_Manager;
use std::time::Duration;

pub fn update(dt: &Duration, em: &mut Entity_Manager) {
    let dt_secs = time::to_secs_frac(&dt);
    let animated_sprites = em.get_component_tuple_mut::<C_Renderable, C_Animated_Sprite>();

    for (renderable, anim_sprite) in animated_sprites.filter(|(_, sprite)| {
        let sprite = sprite.borrow();
        sprite.frame_time > 0.0 && sprite.n_frames > 1
    }) {
        anim_sprite.borrow_mut().frame_time_elapsed += dt_secs;

        let (frame_time, frame_time_elapsed) = {
            let sprite = anim_sprite.borrow();
            (sprite.frame_time, sprite.frame_time_elapsed)
        };
        if frame_time_elapsed >= frame_time {
            anim_sprite.borrow_mut().frame_time_elapsed = 0.0;

            let rect = renderable.borrow().rect;
            let width = rect.width();
            let x =
                (rect.x() + width as i32) % (width * (anim_sprite.borrow().n_frames - 1)) as i32;

            renderable.borrow_mut().rect.set_x(x);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::common::rect::Rect;
    use crate::core::env::Env_Info;
    use crate::gfx;
    use crate::resources::{self, tex_path, Resources};
    use crate::test_common;

    #[test]
    fn animation_system() {
        let (loaders, _, _) = test_common::create_resource_loaders();
        let (mut rsrc, env) = test_common::create_test_resources_and_env(&loaders);
        let mut em = Entity_Manager::new();
        em.register_component::<C_Renderable>();
        em.register_component::<C_Animated_Sprite>();

        let e = em.new_entity();
        {
            let mut r = em.add_component::<C_Renderable>(e);
            r.texture = rsrc.load_texture(&tex_path(&env, "plant.png"));
            r.rect = Rect::new(0, 0, 96, 96);
        }
        {
            let mut a = em.add_component::<C_Animated_Sprite>(e);
            a.n_frames = 4;
            a.frame_time = 0.1;
        }

        let mut dt = Duration::from_millis(16);
        for i in 0..1000 {
            update(&dt, &mut em);
            let r = em.get_component::<C_Renderable>(e).unwrap();
            assert!(
                r.rect.x() % r.rect.width() as i32 == 0,
                "sprite from spritesheet is not aligned! (x = {} not multiple of sprite width {})",
                r.rect.x(),
                r.rect.width()
            );

            if i % 50 == 0 && i > 0 {
                dt = Duration::from_millis((i / 50) * (i / 50) * 2);
            }
        }
    }
}
