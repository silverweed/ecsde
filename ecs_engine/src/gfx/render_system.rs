use crate::common::colors::Color;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::components::gfx::{C_Camera2D, C_Renderable};
use crate::ecs::ecs_world::{Ecs_World, Entity};
use crate::ecs::entity_stream::new_entity_stream;
use crate::gfx::render;
use crate::gfx::window;
use crate::resources;

#[derive(Copy, Clone)]
pub struct Render_System_Config {
    pub clear_color: Color,
    #[cfg(debug_assertions)]
    pub draw_sprites_bg: bool,
    #[cfg(debug_assertions)]
    pub draw_sprites_bg_color: Color,
}

pub struct Render_System_Update_Args<'a> {
    pub window: &'a mut window::Window_Handle,
    pub resources: &'a resources::gfx::Gfx_Resources<'a>,
    pub batches: &'a mut render::batcher::Batches,
    pub camera: &'a C_Camera2D,
    pub ecs_world: &'a Ecs_World,
    pub cfg: Render_System_Config,
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
            batches,
            camera,
            ecs_world,
            cfg,
        } = args;

        trace!("render_system::update");

        window::set_clear_color(window, cfg.clear_color);
        window::clear(window);

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

            //let texture = resources.get_texture(*tex_id);
            let rend_transform = spatial.global_transform;

            //#[cfg(debug_assertions)]
            //{
            //if cfg.draw_sprites_bg {
            //gfx::render::fill_color_rect_ws(
            //window,
            //cfg.draw_sprites_bg_color,
            //gfx::render::sprite_global_bounds(&sprite),
            //&rend_transform,
            //&camera.transform,
            //);
            //}
            //}

            {
                render::render_texture_ws(
                    window,
                    batches,
                    *tex_id,
                    src_rect,
                    *modulate,
                    &rend_transform,
                );
            }
        }
    }
}
