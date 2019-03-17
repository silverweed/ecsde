use crate::core;
use crate::core::common;
use crate::core::resources::Resources;
use crate::ecs::components as comp;
use crate::ecs::entity_manager::{Entity, Entity_Manager};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Gameplay_System {
    entity_manager: Entity_Manager,
}

impl core::system::System for Gameplay_System {
    type Config = ();
    type Update_Params = ();

    fn init(&mut self, cfg: Self::Config) -> common::Maybe_Error {
        self.register_all_components();

        // #DEMO

        // #DEMO

        Ok(())
    }

    fn update(&mut self, params: Self::Update_Params) {}
}

impl Gameplay_System {
    pub fn new() -> Gameplay_System {
        Gameplay_System {
            entity_manager: Entity_Manager::new(),
        }
    }

    fn register_all_components(&mut self) {
        let em = &mut self.entity_manager;

        em.register_component::<comp::C_Position2D>();
        em.register_component::<comp::C_Renderable>();
    }
}
