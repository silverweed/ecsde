use crate::core::common::colors::Color;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::components::gfx::{C_Camera2D, C_Renderable};
use crate::ecs::ecs_world::{Ecs_World, Entity};
use crate::ecs::entity_stream::new_entity_stream;
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
    pub ecs_world: &'a Ecs_World,
    pub frame_lag_normalized: f32,
    pub cfg: Render_System_Config,
    pub dt: std::time::Duration,
    pub _tracer: Debug_Tracer,
}

pub struct Render_System {
    entities_buf: Vec<Entity>,
}

impl Render_System {
    pub fn new() -> Render_System {
        Render_System {
            entities_buf: vec![],
        }
    }

    pub fn update(&mut self, args: Render_System_Update_Args) {
        let Render_System_Update_Args {
            window,
            resources,
            camera,
            ecs_world,
            frame_lag_normalized,
            cfg,
            dt,
            _tracer,
        } = args;

        trace!("render_system::update", _tracer);

        gfx::window::set_clear_color(window, cfg.clear_color);
        gfx::window::clear(window);

        self.entities_buf.clear();
        new_entity_stream(ecs_world)
            .require::<C_Renderable>()
            .require::<C_Spatial2D>()
            .build()
            .collect(ecs_world, &mut self.entities_buf);

        let map_renderable = ecs_world.get_components_map::<C_Renderable>();
        let map_spatial = ecs_world.get_components_map::<C_Spatial2D>();

        for &entity in &self.entities_buf {
            let rend = map_renderable.get_component(entity).unwrap();
            let spatial = map_spatial.get_component(entity).unwrap();

            let C_Renderable {
                texture: tex_id,
                rect: src_rect,
                modulate,
            } = rend;

            let texture = resources.get_texture(*tex_id);
            let mut sprite = gfx::render::create_sprite(texture, src_rect);

            let mut rend_transform = spatial.global_transform;
            if cfg.smooth_by_extrapolating_velocity {
                let v = spatial.velocity * crate::core::time::to_secs_frac(&dt);
                rend_transform.translate(v.x * frame_lag_normalized, v.y * frame_lag_normalized);
            }

            #[cfg(debug_assertions)]
            {
                if cfg.draw_sprites_bg {
                    gfx::render::fill_color_rect_ws(
                        window,
						cfg.draw_sprites_bg_color,
                        gfx::render::sprite_global_bounds(&sprite),
                        &rend_transform,
                        &camera.transform,
                    );
                }
            }

            gfx::render::set_sprite_modulate(&mut sprite, *modulate);
            {
                trace!("render_system::render_sprite", _tracer);
                gfx::render::render_sprite(window, &mut sprite, &rend_transform, &camera.transform);
            }
        }
    }
}
