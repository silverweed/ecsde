use inle_alloc::temp;
use inle_common::colors::Color;
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_query::Ecs_Query;
use inle_ecs::ecs_world::{Component_Storage_Read, Ecs_World, Entity};
use inle_gfx::components::{C_Multi_Renderable, C_Renderable};
use inle_gfx::material::Material;
use inle_gfx::render::batcher::Batches;
use inle_gfx::render::{self, Z_Index};
use inle_gfx::render_window::Render_Window_Handle;
use inle_math::transform::Transform2D;
use inle_resources::gfx::{Gfx_Resources, Shader_Cache};

#[cfg(debug_assertions)]
use {inle_common::colors, inle_math::rect::Rect};

#[derive(Copy, Clone)]
pub struct Render_System_Config {
    pub clear_color: Color,

    #[cfg(debug_assertions)]
    pub debug_visualization: Debug_Visualization,
}

#[cfg(debug_assertions)]
#[derive(Copy, Clone)]
#[non_exhaustive]
pub enum Debug_Visualization {
    None,
    Sprites_Boundaries,
    Normals,
    Materials,
}

pub struct Render_System_Update_Args<'a> {
    pub window: &'a mut Render_Window_Handle,
    pub batches: &'a mut Batches,
    pub ecs_world: &'a Ecs_World,
    pub frame_alloc: &'a mut temp::Temp_Allocator,
    pub cfg: Render_System_Config,
    pub camera: &'a Transform2D,
    pub gres: &'a Gfx_Resources<'a>,
    pub shader_cache: &'a Shader_Cache<'a>,
}

pub fn update(args: Render_System_Update_Args) {
    let Render_System_Update_Args {
        batches,
        ecs_world,
        frame_alloc,
        cfg,
        window,
        gres,
        shader_cache,
        ..
    } = args;

    trace!("render_system::update");

    ////
    //// Renderables
    ////
    {
        trace!("draw_renderables");

        let mut query = Ecs_Query::new(ecs_world)
            .read::<C_Spatial2D>()
            .read::<C_Renderable>();
        let storages = query.storages();
        let spatials = storages.begin_read::<C_Spatial2D>();
        let renderables = storages.begin_read::<C_Renderable>();
        {
            #[cfg(debug_assertions)]
            let (min_z, max_z) = get_min_max_z(query.entities(), &renderables);

            for &entity in query.entities() {
                let rend = renderables.must_get(entity);
                let spatial = spatials.must_get(entity);

                let C_Renderable {
                    material,
                    rect: src_rect,
                    modulate,
                    z_index,
                    sprite_local_transform,
                } = rend;

                let visual_transform = spatial.transform.combine(sprite_local_transform);
                let mut_in_debug!(material) = *material;

                #[cfg(debug_assertions)]
                {
                    display_debug_visualization(
                        window,
                        batches,
                        gres,
                        shader_cache,
                        spatial,
                        &mut material,
                        src_rect,
                        *z_index,
                        min_z,
                        max_z,
                        cfg.debug_visualization,
                    );
                }

                render::render_texture_ws(
                    window,
                    batches,
                    &material,
                    src_rect,
                    *modulate,
                    &visual_transform,
                    *z_index,
                );
            }
        }
    }

    ////
    //// Multi_Renderables
    ////

    {
        trace!("draw_multi_renderables");

        let mut query = Ecs_Query::new(ecs_world)
            .read::<C_Spatial2D>()
            .read::<C_Multi_Renderable>();
        let storages = query.storages();
        let spatials = storages.begin_read::<C_Spatial2D>();
        let multi_renderables = storages.begin_read::<C_Multi_Renderable>();

        #[cfg(debug_assertions)]
        let (min_z, max_z) = get_min_max_z_multi(query.entities(), &multi_renderables);

        for &entity in query.entities() {
            let rend = multi_renderables.must_get(entity);
            let spatial = spatials.must_get(entity);

            let C_Multi_Renderable {
                renderables,
                n_renderables,
            } = rend;

            for i in 0..*n_renderables {
                let C_Renderable {
                    material,
                    rect: src_rect,
                    modulate,
                    z_index,
                    sprite_local_transform,
                } = &renderables[i as usize];

                let transform = spatial.transform.combine(sprite_local_transform);
                let mut_in_debug!(material) = *material;

                #[cfg(debug_assertions)]
                {
                    display_debug_visualization(
                        window,
                        batches,
                        gres,
                        shader_cache,
                        spatial,
                        &mut material,
                        src_rect,
                        *z_index,
                        min_z,
                        max_z,
                        cfg.debug_visualization,
                    );
                }

                render::render_texture_ws(
                    window, batches, &material, src_rect, *modulate, &transform, *z_index,
                );
            }
        }
    }
}

#[cfg(debug_assertions)]
fn get_min_max_z(
    entities: &[Entity],
    renderables: &Component_Storage_Read<C_Renderable>,
) -> (render::Z_Index, render::Z_Index) {
    trace!("get_min_max_z");

    let mut min_z = render::Z_Index::MAX;
    let mut max_z = render::Z_Index::MIN;
    for &entity in entities {
        let C_Renderable { z_index, .. } = renderables.must_get(entity);

        let z_index = *z_index;

        if z_index < min_z {
            min_z = z_index;
        }

        if z_index > max_z {
            max_z = z_index;
        }
    }

    (min_z, max_z)
}

#[cfg(debug_assertions)]
fn get_min_max_z_multi(
    entities: &[Entity],
    multi_renderables: &Component_Storage_Read<C_Multi_Renderable>,
) -> (render::Z_Index, render::Z_Index) {
    trace!("get_min_max_z_multi");

    let mut min_z = render::Z_Index::MAX;
    let mut max_z = render::Z_Index::MIN;
    for &entity in entities {
        let C_Multi_Renderable { renderables, .. } = multi_renderables.must_get(entity);

        for rend in renderables {
            let z_index = rend.z_index;

            if z_index < min_z {
                min_z = z_index;
            }

            if z_index > max_z {
                max_z = z_index;
            }
        }
    }

    (min_z, max_z)
}

#[cfg(debug_assertions)]
fn display_debug_visualization(
    window: &mut Render_Window_Handle,
    batches: &mut Batches,
    gres: &Gfx_Resources,
    shader_cache: &Shader_Cache,
    spatial: &C_Spatial2D,
    material: &mut Material,
    src_rect: &Rect<i32>,
    z_index: Z_Index,
    min_z: Z_Index,
    max_z: Z_Index,
    debug_visualization: Debug_Visualization,
) {
    match debug_visualization {
        Debug_Visualization::Sprites_Boundaries => {
            let mat = Material::with_texture(gres.get_white_texture_handle());
            let color = colors::lerp_col(
                colors::RED,
                colors::AQUA,
                (z_index - min_z) as f32 / (max_z - min_z) as f32,
            );
            render::render_texture_ws(
                window,
                batches,
                &mat,
                src_rect,
                color,
                &spatial.transform,
                z_index,
            );
        }
        Debug_Visualization::Normals => {
            let mut mat = Material::with_texture(if material.normals.is_some() {
                material.normals
            } else {
                gres.get_white_texture_handle()
            });
            mat.shader = shader_cache.get_basic_batcher_shader_handle();
            mat.cast_shadows = false;
            *material = mat;
        }
        _ => {}
    }
}
