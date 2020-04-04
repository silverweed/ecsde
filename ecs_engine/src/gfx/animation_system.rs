use crate::ecs::components::gfx::{C_Animated_Sprite, C_Renderable};
use crate::ecs::ecs_world::Ecs_World;
use std::time::Duration;

pub fn update(dt: &Duration, ecs_world: &mut Ecs_World) {
    let dt_secs = dt.as_secs_f32();

    foreach_entity!(ecs_world, +C_Renderable, +C_Animated_Sprite, |entity| {
        let sprite = ecs_world
            .get_component_mut::<C_Animated_Sprite>(entity)
            .unwrap();
        if sprite.frame_time <= 0.0 || sprite.n_frames <= 1 {
            return;
        }
        sprite.frame_time_elapsed += dt_secs;

        let C_Animated_Sprite {
            frame_time,
            frame_time_elapsed,
            n_frames,
            ..
        } = *sprite;

        if frame_time_elapsed >= frame_time {
            sprite.frame_time_elapsed = 0.0;

            let renderable = ecs_world.get_component_mut::<C_Renderable>(entity).unwrap();
            let rect = renderable.rect;
            let width = rect.width;
            let x = (rect.x + width) % (width * (n_frames) as i32) as i32;

            renderable.rect.x = x;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::rect::Rect;
    use crate::resources::gfx::tex_path;
    use crate::test_common;

    #[test]
    fn animation_system() {
        let (mut gres, _, env) = test_common::create_test_resources_and_env();
        let mut ecs_world = Ecs_World::new();
        ecs_world.register_component::<C_Renderable>();
        ecs_world.register_component::<C_Animated_Sprite>();

        let e = ecs_world.new_entity();
        {
            let mut r = ecs_world.add_component(e, C_Renderable::default());
            r.texture = gres.load_texture(&tex_path(&env, "plant.png"));
            r.rect = Rect::new(0, 0, 96, 96);
        }
        {
            ecs_world.add_component(
                e,
                C_Animated_Sprite {
                    n_frames: 4,
                    frame_time: 0.1,
                    ..Default::default()
                },
            );
        }

        let mut dt = Duration::from_millis(16);
        for i in 0..1000 {
            update(&dt, &mut ecs_world);
            let r = ecs_world.get_component::<C_Renderable>(e).unwrap();
            assert!(
                r.rect.x % r.rect.width as i32 == 0,
                "sprite from spritesheet is not aligned! (x = {} not multiple of sprite width {})",
                r.rect.x,
                r.rect.width
            );

            if i % 50 == 0 && i > 0 {
                dt = Duration::from_millis((i / 50) * (i / 50) * 2);
            }
        }
    }
}
