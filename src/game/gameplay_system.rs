use super::controllable_system::C_Controllable;
use crate::cfg;
use crate::core::common;
use crate::core::common::rect::Rect;
use crate::core::env::Env_Info;
use crate::core::input;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::components::gfx::{C_Animated_Sprite, C_Renderable};
use crate::ecs::entity_manager::{Entity, Entity_Manager};
use crate::game;
use crate::gfx;
use std::cell::Ref;
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

    pub fn init(&mut self, cfg: &cfg::Config) -> common::Maybe_Error {
        self.register_all_components();

        self.init_demo_sprites(cfg);

        Ok(())
    }

    pub fn update(&mut self, dt: &Duration, actions: &input::Action_List) {
        ///// Update all game systems /////
        gfx::animation_system::update(&dt, &mut self.entity_manager);
        game::controllable_system::update(&dt, actions, &mut self.entity_manager);
    }

    pub fn get_renderable_entities(&self) -> Vec<(Ref<'_, C_Renderable>, Ref<'_, C_Spatial2D>)> {
        self.entity_manager
            .get_component_tuple::<C_Renderable, C_Spatial2D>()
            .collect()
    }

    fn register_all_components(&mut self) {
        let em = &mut self.entity_manager;

        em.register_component::<C_Spatial2D>();
        em.register_component::<C_Renderable>();
        em.register_component::<C_Animated_Sprite>();
        em.register_component::<C_Controllable>();
    }

    // #DEMO
    fn init_demo_sprites(&mut self, cfg: &cfg::Config) {
        let em = &mut self.entity_manager;
        let yv = em.new_entity();
        self.entities.push(yv);
        {
            let mut s = em.add_component::<C_Spatial2D>(yv);
            s.transform.set_position(300.0, 200.0);
            s.transform.set_scale(3.0, 3.0);
        }
        //{
        //let mut rend = em.add_component::<C_Renderable>(yv);
        //rend.texture = rsrc.load_texture(&tex_path(&env, "yv.png"));
        //assert!(rend.texture.is_some(), "Could not load yv texture!");
        //rend.rect = Rect::new(0, 0, 148, 125);
        //}

        let plant = em.new_entity();
        self.entities.push(plant);
        {
            let mut s = em.add_component::<C_Spatial2D>(plant);
            s.transform.set_position(400.0, 500.0);
        }
        //{
        //let mut rend = em.add_component::<C_Renderable>(plant);
        //rend.texture = rsrc.load_texture(&tex_path(&env, "plant.png"));
        //assert!(rend.texture.is_some(), "Could not load plant texture!");
        //rend.rect = Rect::new(0, 0, 96, 96);
        //}
        {
            let mut asprite = em.add_component::<C_Animated_Sprite>(plant);
            asprite.n_frames = 4;
            asprite.frame_time = 0.1;
        }
        {
            let mut ctrl = em.add_component::<C_Controllable>(plant);
            ctrl.speed = cfg.get_var_float_or("gameplay/player/player_speed", 300.0);
        }
    }
}
