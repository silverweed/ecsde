use inle_ecs::ecs_query_new::Ecs_Query;
use inle_ecs::ecs_world::System;

pub struct Test_System {}

impl System for Test_System {
    fn get_queries_mut(&mut self) -> &mut [Ecs_Query] {}
}
