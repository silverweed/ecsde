use super::interface::{Game_System, Update_Args};
use inle_ecs::ecs_query_new::Ecs_Query;

pub mod test_ai_system;

#[derive(Default)]
pub struct Ai_System {}

impl Game_System for Ai_System {
    fn get_queries_mut(&mut self) -> Vec<&mut Ecs_Query> {
        vec![]
    }

    fn update(&self, args: &mut Update_Args) {
        test_ai_system::update(args.ecs_world, args.phys_world, &args.engine_state.config);
    }
}
