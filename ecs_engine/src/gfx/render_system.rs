use crate::alloc::temp;
use crate::common::colors::Color;
use crate::common::transform::Transform2D;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::components::gfx::{C_Multi_Renderable, C_Renderable};
use crate::ecs::ecs_world::Ecs_World;
use crate::ecs::entity_stream::new_entity_stream;
use crate::gfx::render;
use crate::gfx::render_window::Render_Window_Handle;

#[derive(Copy, Clone)]
pub struct Render_System_Config {
    pub clear_color: Color,
    #[cfg(debug_assertions)]
    pub draw_sprites_bg: bool,
    #[cfg(debug_assertions)]
    pub draw_sprites_bg_color: Color,
}

pub struct Render_System_Update_Args<'a> {
    pub window: &'a mut Render_Window_Handle,
    pub batches: &'a mut render::batcher::Batches,
    pub ecs_world: &'a Ecs_World,
    pub frame_alloc: &'a mut temp::Temp_Allocator,
    pub cfg: Render_System_Config,
    pub camera: &'a Transform2D,
}

pub fn update(args: Render_System_Update_Args) {
    let Render_System_Update_Args {
        batches,
        ecs_world,
        frame_alloc,
        cfg,
        window,
        camera,
    } = args;

    trace!("render_system::update");

    let renderables = ecs_world.get_component_storage::<C_Renderable>();
    let spatials = ecs_world.get_component_storage::<C_Spatial2D>();

    {
        let mut entities = temp::excl_temp_array(frame_alloc);
        new_entity_stream(ecs_world)
            .require::<C_Renderable>()
            .require::<C_Spatial2D>()
            .build()
            .collect(ecs_world, &mut entities);

        for &entity in entities.as_slice() {
            let rend = renderables.get_component(entity).unwrap();
            let spatial = spatials.get_component(entity).unwrap();

            let C_Renderable {
                material,
                rect: src_rect,
                modulate,
                z_index,
            } = rend;

            #[cfg(debug_assertions)]
            {
                // @Fixme: not working
                if cfg.draw_sprites_bg {
                    render::render_rect_ws(
                        window,
                        *src_rect,
                        cfg.draw_sprites_bg_color,
                        &spatial.transform,
                        camera,
                    );
                }
            }

            render::render_texture_ws(
                batches,
                *material,
                src_rect,
                *modulate,
                &spatial.transform,
                *z_index,
            );
        }
    }

    let mut entities = temp::excl_temp_array(frame_alloc);
    new_entity_stream(ecs_world)
        .require::<C_Multi_Renderable>()
        .require::<C_Spatial2D>()
        .build()
        .collect(ecs_world, &mut entities);

    let multi_renderables = ecs_world.get_component_storage::<C_Multi_Renderable>();

    for &entity in entities.as_slice() {
        let rend = multi_renderables.get_component(entity).unwrap();
        let spatial = spatials.get_component(entity).unwrap();

        let C_Multi_Renderable {
            renderables,
            rend_transforms,
            n_renderables,
        } = rend;

        //#[cfg(debug_assertions)]
        //{
        //if cfg.draw_sprites_bg {
        //render::render_rect_ws(
        //window,
        //*src_rect,
        //cfg.draw_sprites_bg_color,
        //&spatial.transform,
        //camera,
        //);
        //}
        //}

        for i in 0..*n_renderables {
            let C_Renderable {
                material,
                rect: src_rect,
                modulate,
                z_index,
            } = &renderables[i as usize];
            let rend_transform = &rend_transforms[i as usize];

            let transform = spatial.transform.combine(rend_transform);
            render::render_texture_ws(
                batches, *material, src_rect, *modulate, &transform, *z_index,
            );
        }
    }
}
