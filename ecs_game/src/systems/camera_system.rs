use super::interface::{Game_System, Realtime_Update_Args};
use inle_cfg::{Cfg_Var, Config};
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_query_new::Ecs_Query;
use inle_ecs::ecs_world::Entity;
use inle_gfx::components::C_Camera2D;
use inle_math::vector::Vec2f;

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

pub struct Camera_System {
    query: Ecs_Query,
    camera_on_player: Cfg_Var<bool>,
}

impl Camera_System {
    pub fn new(cfg: &Config) -> Self {
        Self {
            query: Ecs_Query::default()
                .require::<C_Camera_Follow>()
                .require::<C_Camera2D>(),
            camera_on_player: Cfg_Var::new("game/camera/on_player", cfg),
        }
    }
}

impl Game_System for Camera_System {
    fn get_queries_mut(&mut self) -> Vec<&mut Ecs_Query> {
        vec![&mut self.query]
    }

    fn realtime_update(&self, args: &mut Realtime_Update_Args) {
        let Realtime_Update_Args {
            ecs_world: world,
            dt,
            engine_state,
            ..
        } = args;

        let cfg = &engine_state.config;

        if !self.camera_on_player.read(cfg) {
            return;
        }

        foreach_entity!(self.query, world,
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
}
