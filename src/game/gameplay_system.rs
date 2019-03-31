use crate::core::common;
use crate::core::env::Env_Info;
use crate::ecs::components as comp;
use crate::ecs::entity_manager::{Entity, Entity_Manager};
use crate::resources::resources::{tex_path, Resources};

pub struct Gameplay_System {
    entity_manager: Entity_Manager,
    entities: Vec<Entity>,
}

impl Gameplay_System {
    pub fn new() -> Gameplay_System {
        Gameplay_System {
            entity_manager: Entity_Manager::new(),
            entities: vec![],
        }
    }

    pub fn init(&mut self, env: &Env_Info, rsrc: &mut Resources) -> common::Maybe_Error {
        self.register_all_components();

        // #DEMO
        let em = &mut self.entity_manager;
        let e = em.new_entity();
        self.entities.push(e);
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

    pub fn get_renderable_entities(&self) -> Vec<(&comp::C_Renderable, &comp::C_Position2D)> {
        self.entities
            .iter()
            .map(|&e| {
                (
                    self.entity_manager.get_component::<comp::C_Renderable>(e),
                    self.entity_manager.get_component::<comp::C_Position2D>(e),
                )
            })
            .filter(|(r, p)| r.is_some() && p.is_some())
            .map(|(r, p)| (r.unwrap(), p.unwrap()))
            .collect()
    }

    fn register_all_components(&mut self) {
        let em = &mut self.entity_manager;

        em.register_component::<comp::C_Position2D>();
        em.register_component::<comp::C_Renderable>();
    }
}
