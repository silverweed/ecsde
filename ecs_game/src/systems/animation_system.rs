use super::interface::{Game_System, Update_Args};
use inle_ecs::ecs_query_new::Ecs_Query;
use inle_gfx::components::{C_Animated_Sprite, C_Renderable};

pub struct Animation_System {
    query: Ecs_Query,
}

impl Animation_System {
    pub fn new() -> Self {
        Self {
            query: Ecs_Query::default()
                .require::<C_Renderable>()
                .require::<C_Animated_Sprite>(),
        }
    }
}

impl Game_System for Animation_System {
    fn get_queries_mut(&mut self) -> Vec<&mut Ecs_Query> {
        vec![&mut self.query]
    }

    fn update(&self, args: &mut Update_Args) {
        let dt_secs = args.dt.as_secs_f32();

        foreach_entity!(self.query, args.ecs_world,
            read: ;
            write: C_Renderable, C_Animated_Sprite;
            |_e, (), (renderable, sprite): (&mut C_Renderable, &mut C_Animated_Sprite)| {
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

                let rect = renderable.rect;
                let width = rect.width;
                let x = (rect.x + width) % (width * n_frames as i32).max(1);

                renderable.rect.x = x;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inle_math::rect::Rect;
    use inle_resources::gfx::tex_path;
    use inle_test::test_common;
    use serial_test::serial;

    #[test]
    #[serial]
    fn animation_system() {
        let (_win, _glfw) = test_common::load_gl_pointers();
        let (mut gres, _, env) = test_common::create_test_resources_and_env();
        let mut ecs_world = Ecs_World::new();

        let e = ecs_world.new_entity();
        {
            let mut r = C_Renderable::default();
            r.material.texture = gres.load_texture(&tex_path(&env, "ground.png"));
            r.rect = Rect::new(0, 0, 96, 96);
            ecs_world.add_component(e, r);
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
