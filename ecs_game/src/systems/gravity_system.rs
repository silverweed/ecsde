use inle_ecs::ecs_world::Ecs_World;
use inle_ecs::components::base::C_Spatial2D;
use inle_cfg::{Config, var::Cfg_Var};
use std::time::Duration;

#[derive(Copy, Clone, Debug)]
pub struct C_Gravity {
    // This should be positive for a downward gravity
    pub acceleration: Cfg_Var<f32>,
}

pub fn update(dt: &Duration, world: &mut Ecs_World, cfg: &Config) {
    let secs = dt.as_secs_f32();
    foreach_entity!(world, +C_Spatial2D, +C_Gravity, |entity| {
        let gravity = world.get_component::<C_Gravity>(entity).unwrap().acceleration.read(cfg);
        let spatial = world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        spatial.velocity += secs * v2!(0.0, gravity);
    });
}
