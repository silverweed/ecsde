use crate::core::time;
use crate::ecs::components as comp;
use std::cell::RefCell;
use std::time::Duration;

pub fn update_animated_sprites(dt: &Duration, animated_sprites: &mut [&mut comp::C_Renderable]) {
    let dt_secs = time::to_secs_frac(&dt);

    for sprite in animated_sprites {
        if sprite.frame_time <= 0.0 || sprite.n_frames < 2 {
            continue;
        }

        sprite.frame_time_elapsed += dt_secs;

        if sprite.frame_time_elapsed >= sprite.frame_time {
            sprite.frame_time_elapsed = 0.0;

            let width = sprite.rect.width();
            let x = if sprite.rect.x == (width * (sprite.n_frames - 1)) as i32 {
                0
            } else {
                sprite.rect.x + width as i32
            };

            sprite.rect.x = x;
        }
    }
}
