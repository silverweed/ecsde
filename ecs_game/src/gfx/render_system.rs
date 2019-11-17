use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::components::gfx::{C_Camera2D, C_Renderable};
use ecs_engine::core::common::colors::{self, Color};
use ecs_engine::core::common::Maybe_Error;
use ecs_engine::gfx as ngfx;
use ecs_engine::resources;
use std::cell::Ref;

pub struct Render_System_Config {
    pub clear_color: Color,
}

pub struct Render_System {
    config: Render_System_Config,
}

impl Render_System {
    pub fn new() -> Self {
        Render_System {
            config: Self::default_config(),
        }
    }

    fn default_config() -> Render_System_Config {
        Render_System_Config {
            clear_color: colors::rgb(0, 0, 0),
        }
    }

    pub fn init(&mut self, cfg: Render_System_Config) -> Maybe_Error {
        self.config = cfg;
        Ok(())
    }

    pub fn update(
        &mut self,
        window: &mut ngfx::window::Window_Handle,
        resources: &resources::gfx::Gfx_Resources,
        camera: &C_Camera2D,
        renderables: &[(Ref<'_, C_Renderable>, Ref<'_, C_Spatial2D>)],
        frame_lag_normalized: f32,
        smooth_by_extrapolating_velocity: bool,
    ) {
        ngfx::window::set_clear_color(window, self.config.clear_color);
        ngfx::window::clear(window);

        for (rend, spatial) in renderables {
            let rend: &C_Renderable = &*rend;
            let C_Renderable {
                texture: tex_id,
                rect: src_rect,
                ..
            } = rend;

            let texture = resources.get_texture(*tex_id);
            let sprite = ngfx::render::create_sprite(texture, *src_rect);

            let mut rend_transform = spatial.global_transform;
            if smooth_by_extrapolating_velocity {
                let v = spatial.velocity;
                rend_transform.translate(v.x * frame_lag_normalized, v.y * frame_lag_normalized);
            }

            ngfx::render::render_sprite(window, &sprite, &rend_transform, &camera.transform);
        }
    }
}