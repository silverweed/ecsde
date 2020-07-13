use crate::entities;
use ecs_engine::cfg::Config;
use ecs_engine::collisions::phys_world::Physics_World;
use ecs_engine::common::stringid::String_Id;
use ecs_engine::common::transform::Transform2D;
use ecs_engine::core::env::Env_Info;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::ecs_world::Ecs_World;
use ecs_engine::gfx::render_window::{mouse_pos_in_world, Render_Window_Handle};
use ecs_engine::input::input_state::{Action_Kind, Game_Action};
use ecs_engine::resources::gfx::{Gfx_Resources, Shader_Cache};

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Entity_Preview;

pub fn update(
    world: &mut Ecs_World,
    phys_world: &mut Physics_World,
    window: &Render_Window_Handle,
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    camera: &Transform2D,
    actions: &[Game_Action],
    cfg: &Config,
) {
    foreach_entity!(world, +C_Spatial2D, +C_Entity_Preview, |entity| {
        let spatial = world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        let mpos = mouse_pos_in_world(window, camera);
        spatial.transform.set_position_v(mpos);

        if actions.contains(&(String_Id::from("place_entity"), Action_Kind::Pressed)) {
            let transform = spatial.transform;
            // @Temporary
            entities::create_drill(world, phys_world, gres, shader_cache, env, cfg, &transform);
        }
    });
}
