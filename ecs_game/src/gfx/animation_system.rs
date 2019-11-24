use crate::ecs::components::gfx::{C_Animated_Sprite, C_Renderable};
use crate::ecs::entity_manager::Ecs_World;
use crate::ecs::entity_stream::Entity_Stream;
use ecs_engine::core::time;
use std::time::Duration;

pub fn update(dt: &Duration, ecs_world: &mut Ecs_World, mut entity_stream: Entity_Stream) {
    let dt_secs = time::to_secs_frac(&dt);

    loop {
        let entity = entity_stream.next(ecs_world);
        if entity.is_none() {
            break;
        }
        let entity = entity.unwrap();

        let sprite = ecs_world
            .get_component_mut::<C_Animated_Sprite>(entity)
            .unwrap();
        if sprite.frame_time <= 0.0 || sprite.n_frames <= 1 {
            continue;
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
            let width = rect.width();
            let x = (rect.x() + width) % (width * (n_frames - 1) as i32) as i32;

            renderable.rect.set_x(x);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::entity_stream::new_entity_stream;
    use ecs_engine::core::common::rect::Rect;
    use ecs_engine::resources::gfx::tex_path;
    use ecs_engine::test_common;

    #[test]
    fn animation_system() {
        let (mut gres, _, env) = test_common::create_test_resources_and_env();
        let mut ecs_world = Ecs_World::new();
        ecs_world.register_component::<C_Renderable>();
        ecs_world.register_component::<C_Animated_Sprite>();

        let e = ecs_world.new_entity();
        {
            let mut r = ecs_world.add_component::<C_Renderable>(e);
            r.texture = gres.load_texture(&tex_path(&env, "plant.png"));
            r.rect = Rect::new(0, 0, 96, 96);
        }
        {
            let mut a = ecs_world.add_component::<C_Animated_Sprite>(e);
            a.n_frames = 4;
            a.frame_time = 0.1;
        }

        let mut dt = Duration::from_millis(16);
        for i in 0..1000 {
            let stream = new_entity_stream(&ecs_world)
                .require::<C_Renderable>()
                .require::<C_Animated_Sprite>()
                .build();
            update(&dt, &mut ecs_world, stream);
            let r = ecs_world.get_component::<C_Renderable>(e).unwrap();
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
