use crate::entities;
use inle_cfg::Config;
use inle_physics::phys_world::Physics_World;
use inle_common::stringid::String_Id;
use inle_math::transform::Transform2D;
use inle_core::env::Env_Info;
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::Ecs_World;
use inle_gfx::render_window::{mouse_pos_in_world, Render_Window_Handle};
use inle_input::input_state::{Action_Kind, Game_Action};
use inle_resources::gfx::{Gfx_Resources, Shader_Cache};

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
