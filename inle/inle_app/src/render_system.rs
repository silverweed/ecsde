use inle_alloc::temp;
use inle_common::colors::Color;
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::Ecs_World;
use inle_ecs::entity_stream::new_entity_stream;
use inle_gfx::components::{C_Multi_Renderable, C_Renderable};
use inle_gfx::render;
use inle_gfx::render_window::Render_Window_Handle;
use inle_math::transform::Transform2D;
use inle_resources::gfx::Gfx_Resources;

#[cfg(debug_assertions)]
use {
	inle_common::colors
};

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
	Materials
}

pub struct Render_System_Update_Args<'a> {
    pub window: &'a mut Render_Window_Handle,
    pub batches: &'a mut render::batcher::Batches,
    pub ecs_world: &'a Ecs_World,
    pub frame_alloc: &'a mut temp::Temp_Allocator,
    pub cfg: Render_System_Config,
    pub camera: &'a Transform2D,
	pub gres: &'a Gfx_Resources<'a>,
}

pub fn update(args: Render_System_Update_Args) {
    let Render_System_Update_Args {
        batches,
        ecs_world,
        frame_alloc,
        cfg,
        window,
        camera,
		#[cfg(debug_assertions)]
		gres
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

		#[cfg(debug_assertions)]
		let (min_z, max_z) = get_min_max_z(ecs_world, &entities);

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
				let mat = inle_gfx::material::Material::with_texture(gres.get_white_texture_handle());
                if let Debug_Visualization::Sprites_Boundaries = cfg.debug_visualization {
					let color = colors::lerp_col(colors::RED, colors::AQUA, (*z_index - min_z) as f32 / (max_z - min_z) as f32);
                    render::render_texture_ws(
						batches,
						mat,
                        src_rect,
						color,
                        &spatial.transform,
						*z_index,
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

#[cfg(debug_assertions)]
fn get_min_max_z(ecs_world: &Ecs_World, entities: &[inle_ecs::ecs_world::Entity]) -> (render::Z_Index, render::Z_Index) {
	let mut min_z = render::Z_Index::MAX;
	let mut max_z = render::Z_Index::MIN;
	for &entity in entities {
		let C_Renderable {
			z_index,
			..
		} = ecs_world.get_component::<C_Renderable>(entity).unwrap();
		
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
