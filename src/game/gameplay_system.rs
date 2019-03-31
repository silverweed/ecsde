use crate::core::common;
use crate::core::common::vector::Vec2f;
use crate::core::env::Env_Info;
use crate::core::input;
use crate::core::time;
use crate::ecs::components as comp;
use crate::ecs::entity_manager::{Entity, Entity_Manager};
use crate::resources::resources::{tex_path, Resources};
use std::time::Duration;

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
        let yv = em.new_entity();
        self.entities.push(yv);
        {
            let mut pos = em.add_component::<comp::C_Position2D>(yv);
            pos.x = 300.0;
            pos.y = 200.0;
        }
        {
            let mut rend = em.add_component::<comp::C_Renderable>(yv);
            rend.sprite = rsrc.new_sprite(&tex_path(&env, "yv.png"));
        }
        {
            let mut ctrl = em.add_component::<comp::C_Controllable>(yv);
            ctrl.speed = 300.0;
        }

        let plant = em.new_entity();
        self.entities.push(plant);
        {
            let mut pos = em.add_component::<comp::C_Position2D>(plant);
            pos.x = 400.0;
            pos.y = 500.0;
        }
        {
            let mut rend = em.add_component::<comp::C_Renderable>(plant);
            rend.sprite = rsrc.new_sprite(&tex_path(&env, "plant.png"));
        }
        // #DEMO

        Ok(())
    }

    pub fn update(&mut self, dt: &Duration, actions: &input::Action_List) {
        use crate::core::common::direction::Direction;
        use input::Action;

        let em = &mut self.entity_manager;

        let controllable: Vec<&Entity> = self
            .entities
            .iter()
            .filter(|&&e| {
                em.has_component::<comp::C_Position2D>(e)
                    && em.has_component::<comp::C_Controllable>(e)
            })
            .collect();

        let mut movement = Vec2f::new(0.0, 0.0);
        if actions.has_action(&Action::Move(Direction::Left)) {
            movement.x -= 1.0;
        }
        if actions.has_action(&Action::Move(Direction::Right)) {
            movement.x += 1.0;
        }
        if actions.has_action(&Action::Move(Direction::Up)) {
            movement.y -= 1.0;
        }
        if actions.has_action(&Action::Move(Direction::Down)) {
            movement.y += 1.0;
        }

        for &ctrl in controllable {
            let speed = em
                .get_component::<comp::C_Controllable>(ctrl)
                .unwrap()
                .speed;
            let velocity = movement * speed * time::to_secs_frac(dt);
            let pos = em.get_component_mut::<comp::C_Position2D>(ctrl).unwrap();
            pos.x += velocity.x;
            pos.y += velocity.y;
        }
    }

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
        em.register_component::<comp::C_Controllable>();
    }
}
