use inle_cfg::{Cfg_Var, Config};
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::{Ecs_World, Entity};
use inle_gfx::components::C_Camera2D;
use inle_math::vector::Vec2f;
use std::time::Duration;

#[derive(Copy, Clone, Debug)]
pub enum Camera_Follow_Target {
    None,
    Position(Vec2f),
    Entity(Entity),
}

impl Default for Camera_Follow_Target {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Copy, Clone, Default)]
pub struct C_Camera_Follow {
    pub target: Camera_Follow_Target,
    pub lerp_factor: Cfg_Var<f32>,
}

pub fn update(dt: &Duration, world: &mut Ecs_World, cfg: &Config) {
    foreach_entity!(world,
        read: C_Camera_Follow;
        write: C_Camera2D;
        |_e, (cam_follow,): (&C_Camera_Follow,), (camera,): (&mut C_Camera2D,)| {
        let target = cam_follow.target;
        let lerp_factor = cam_follow.lerp_factor.read(cfg);

        let target_pos = match target {
            Camera_Follow_Target::None => { return; },
            Camera_Follow_Target::Position(pos) => pos,
            Camera_Follow_Target::Entity(entity) => {
                let spatial = world.get_component::<C_Spatial2D>(entity).expect("Followed entity has no C_Spatial2D!");
                spatial.transform.position()
            },
        };

        let cam_pos = camera.transform.position();
        let diff = target_pos - cam_pos;
        camera.transform.translate_v(lerp_factor * diff * dt.as_secs_f32());
    });
}
