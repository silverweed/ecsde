use crate::ecs::components::gfx::{C_Camera2D, C_Renderable};
use ecs_engine::core::common::colors::Color;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::ecs_world::Ecs_World;
use ecs_engine::ecs::entity_stream::Entity_Stream;
use ecs_engine::gfx as ngfx;
use ecs_engine::prelude::*;
use ecs_engine::resources;

#[derive(Copy, Clone)]
pub struct Render_System_Config {
    pub clear_color: Color,
    pub smooth_by_extrapolating_velocity: bool,
    #[cfg(debug_assertions)]
    pub draw_sprites_bg: bool,
    #[cfg(debug_assertions)]
    pub draw_sprites_bg_color: Color,
}

pub fn update(
    window: &mut ngfx::window::Window_Handle,
    resources: &resources::gfx::Gfx_Resources,
    camera: &C_Camera2D,
    mut renderables: Entity_Stream,
    ecs_world: &Ecs_World,
    frame_lag_normalized: f32,
    cfg: Render_System_Config,
    tracer: Debug_Tracer,
) {
    trace!("render_system::update", tracer);

    ngfx::window::set_clear_color(window, cfg.clear_color);
    ngfx::window::clear(window);

    loop {
        let entity = renderables.next(ecs_world);
        if entity.is_none() {
            break;
        }
        let entity = entity.unwrap();

        let rend = ecs_world.get_component::<C_Renderable>(entity).unwrap();
        let spatial = ecs_world.get_component::<C_Spatial2D>(entity).unwrap();

        let C_Renderable {
            texture: tex_id,
            rect: src_rect,
            ..
        } = rend;

        let texture = resources.get_texture(*tex_id);
        let sprite = ngfx::render::create_sprite(texture, *src_rect);

        let mut rend_transform = spatial.global_transform;
        if cfg.smooth_by_extrapolating_velocity {
            let v = spatial.velocity;
            rend_transform.translate(v.x * frame_lag_normalized, v.y * frame_lag_normalized);
        }

        #[cfg(debug_assertions)]
        {
            if cfg.draw_sprites_bg {
                ngfx::render::fill_color_rect_ws(
                    window,
                    &ngfx::render::Paint_Properties {
                        color: cfg.draw_sprites_bg_color,
                        ..Default::default()
                    },
                    sprite.global_bounds(),
                    &rend_transform,
                    &camera.transform,
                );
            }
        }
        ngfx::render::render_sprite(window, &sprite, &rend_transform, &camera.transform);
    }
}
