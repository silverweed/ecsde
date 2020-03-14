use crate::alloc::temp;
use crate::common::colors::Color;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::components::gfx::C_Renderable;
use crate::ecs::ecs_world::Ecs_World;
use crate::ecs::entity_stream::new_entity_stream;
use crate::gfx::render;

#[derive(Copy, Clone)]
pub struct Render_System_Config {
    pub clear_color: Color,
    #[cfg(debug_assertions)]
    pub draw_sprites_bg: bool,
    #[cfg(debug_assertions)]
    pub draw_sprites_bg_color: Color,
}

pub struct Render_System_Update_Args<'a> {
    pub batches: &'a mut render::batcher::Batches,
    pub ecs_world: &'a Ecs_World,
    pub frame_alloc: &'a mut temp::Temp_Allocator,
    pub cfg: Render_System_Config,
}

pub fn update(args: Render_System_Update_Args) {
    let Render_System_Update_Args {
        batches,
        ecs_world,
        frame_alloc,
        cfg,
    } = args;

    trace!("render_system::update");

    let mut entities = temp::excl_temp_array(frame_alloc);
    new_entity_stream(ecs_world)
        .require::<C_Renderable>()
        .require::<C_Spatial2D>()
        .build()
        .collect(ecs_world, &mut entities);

    let map_renderable = ecs_world.get_components_map::<C_Renderable>();
    let map_spatial = ecs_world.get_components_map::<C_Spatial2D>();

    for &entity in entities.as_slice() {
        let rend = map_renderable.get_component(entity).unwrap();
        let spatial = map_spatial.get_component(entity).unwrap();

        let C_Renderable {
            texture: tex_id,
            rect: src_rect,
            modulate,
        } = rend;

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

        render::render_texture_ws(
            batches,
            *tex_id,
            src_rect,
            *modulate,
            &spatial.global_transform,
        );
    }
}
