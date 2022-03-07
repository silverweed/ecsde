use crate::systems::interface::{Game_System, Update_Args};
use inle_ecs::ecs_query_new::Ecs_Query;
use inle_ecs::ecs_world::{Component_Manager, Component_Updates, Entity};
use std::collections::HashMap;

pub mod position_history_system;

pub struct Game_Debug_Systems {
    systems: Vec<Box<dyn Game_System>>,
}

impl Game_Debug_Systems {
    pub fn new(cfg: &inle_cfg::Config) -> Self {
        Self {
            systems: vec![Box::new(
                position_history_system::Position_History_System::new(cfg),
            )],
        }
    }

    pub fn update_queries(
        &mut self,
        comp_updates: &HashMap<Entity, Component_Updates>,
        comp_mgr: &Component_Manager,
    ) {
        for (entity, updates) in comp_updates {
            for system in &mut self.systems {
                for query in system.get_queries_mut() {
                    query.update(comp_mgr, *entity, &updates.added, &updates.removed);
                }
            }
        }
    }

    pub fn update(&self, args: &mut Update_Args) {
        for system in &self.systems {
            system.update(args);
        }
    }
}
