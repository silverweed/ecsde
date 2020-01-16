use crate::core::common::colors::Color;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::components::gfx::{C_Camera2D, C_Renderable};
use crate::ecs::ecs_world::Ecs_World;
use crate::ecs::entity_stream::Entity_Stream;
use crate::gfx;
use crate::prelude::*;
use crate::resources;

#[derive(Copy, Clone)]
pub struct Render_System_Config {
    pub clear_color: Color,
    pub smooth_by_extrapolating_velocity: bool,
    #[cfg(debug_assertions)]
    pub draw_sprites_bg: bool,
    #[cfg(debug_assertions)]
    pub draw_sprites_bg_color: Color,
}

pub struct Render_System_Update_Args<'a> {
    pub window: &'a mut gfx::window::Window_Handle,
    pub resources: &'a resources::gfx::Gfx_Resources<'a>,
    pub camera: &'a C_Camera2D,
    pub renderables: Entity_Stream,
    pub ecs_world: &'a Ecs_World,
    pub frame_lag_normalized: f32,
    pub cfg: Render_System_Config,
    pub tracer: Debug_Tracer,
}

pub fn update(args: Render_System_Update_Args) {
    let Render_System_Update_Args {
        window,
        resources,
        camera,
        mut renderables,
        ecs_world,
        frame_lag_normalized,
        cfg,
        tracer,
    } = args;

    trace!("render_system::update", tracer);

    gfx::window::set_clear_color(window, cfg.clear_color);
    gfx::window::clear(window);

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
        let mut sprite = gfx::render::create_sprite(texture, *src_rect);

        let mut rend_transform = spatial.global_transform;
        if cfg.smooth_by_extrapolating_velocity {
            let v = spatial.velocity;
            rend_transform.translate(v.x * frame_lag_normalized, v.y * frame_lag_normalized);
        }

        #[cfg(debug_assertions)]
        {
            if cfg.draw_sprites_bg {
                gfx::render::fill_color_rect_ws(
                    window,
                    &gfx::render::Paint_Properties {
                        color: cfg.draw_sprites_bg_color,
                        ..Default::default()
                    },
                    sprite.global_bounds(),
                    &rend_transform,
                    &camera.transform,
                );
            }
        }
        gfx::render::render_sprite(window, &mut sprite, &rend_transform, &camera.transform);
    }
}
