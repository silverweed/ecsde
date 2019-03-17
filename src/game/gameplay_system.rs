use crate::core::common;
use crate::core::env::Env_Info;
use crate::ecs::components as comp;
use crate::ecs::entity_manager::Entity_Manager;
use crate::resources::resources::{tex_path, Resources};

pub struct Gameplay_System {
    entity_manager: Entity_Manager,
}

impl Gameplay_System {
    pub fn new() -> Gameplay_System {
        Gameplay_System {
            entity_manager: Entity_Manager::new(),
        }
    }

    pub fn init(&mut self, env: &Env_Info, rsrc: &mut Resources) -> common::Maybe_Error {
        self.register_all_components();

        // #DEMO
        let em = &mut self.entity_manager;
        let e = em.new_entity();
        {
            let mut pos = em.add_component::<comp::C_Position2D>(e);
            pos.x = 200.0;
            pos.y = 200.0;
        }
        {
            let mut rend = em.add_component::<comp::C_Renderable>(e);
            rend.sprite = rsrc.new_sprite(&tex_path(&env, "yv.png"));
        }

        // #DEMO

        Ok(())
    }

    pub fn update(&mut self) {}

    fn register_all_components(&mut self) {
        let em = &mut self.entity_manager;

        em.register_component::<comp::C_Position2D>();
        em.register_component::<comp::C_Renderable>();
    }
}
