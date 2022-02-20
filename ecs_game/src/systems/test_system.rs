use super::gravity_system::C_Gravity;
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_query_new::Ecs_Query;
use inle_ecs::ecs_world::{Ecs_World, System};

pub struct Test_System {
    query: Ecs_Query,
}

impl Test_System {
    pub fn new() -> Self {
        let query = Ecs_Query::new().write::<C_Spatial2D>();
        Self { query }
    }

    pub fn update(&mut self, ecs_world: &mut Ecs_World) {
        for entity in self.query.entities() {
            //ldebug!("{:?}", entity);
        }
    }
}

impl System for Test_System {
    fn get_queries_mut(&mut self) -> Vec<&mut Ecs_Query> {
        vec![&mut self.query]
    }
}
