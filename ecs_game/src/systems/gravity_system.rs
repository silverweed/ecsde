use super::ground_detection_system::C_Ground_Detection;
use super::interface::{Game_System, Update_Args};
use inle_cfg::Cfg_Var;
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_query_new::Ecs_Query;

#[derive(Copy, Clone, Debug)]
pub struct C_Gravity {
    // This should be positive for a downward gravity
    pub acceleration: Cfg_Var<f32>,
}

pub struct Gravity_System {
    query: Ecs_Query,
    ground_detect_query: Ecs_Query,
}

impl Gravity_System {
    pub fn new() -> Self {
        Self {
            query: Ecs_Query::default()
                .require::<C_Gravity>()
                .require::<C_Spatial2D>(),
            ground_detect_query: Ecs_Query::default().require::<C_Ground_Detection>(),
        }
    }
}

impl Game_System for Gravity_System {
    fn get_queries_mut(&mut self) -> Vec<&mut Ecs_Query> {
        vec![&mut self.query, &mut self.ground_detect_query]
    }

    fn update(&self, args: &mut Update_Args) {
        foreach_entity!(self.query, args.ecs_world,
            read: C_Gravity;
            write: C_Spatial2D;
        |e, (gravity,): (&C_Gravity,), (spatial,): (&mut C_Spatial2D,)| {
            let touching_ground = if let Some(ground_detect) = args.ecs_world.get_component::<C_Ground_Detection>(e) {
                ground_detect.touching_ground
            } else {
                false
            };
            if !touching_ground {
                spatial.acceleration += v2!(0.0, gravity.acceleration.read(&args.engine_state.config));
            }
        });
    }
}
