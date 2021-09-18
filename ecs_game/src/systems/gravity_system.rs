use inle_cfg::{var::Cfg_Var, Config};
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::Ecs_World;
use std::time::Duration;

#[derive(Copy, Clone, Debug)]
pub struct C_Gravity {
    // This should be positive for a downward gravity
    pub acceleration: Cfg_Var<f32>,
}

pub fn update(dt: &Duration, world: &mut Ecs_World, cfg: &Config) {
    let secs = dt.as_secs_f32();
    foreach_entity_new!(world,
        read: C_Gravity;
        write: C_Spatial2D;
        |entity, (gravity,): (&C_Gravity,), (spatial,): (&mut C_Spatial2D,)| {
        spatial.velocity += secs * v2!(0.0, gravity);
    });
}
