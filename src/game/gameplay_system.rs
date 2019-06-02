use super::controllable_system::C_Controllable;
use crate::cfg;
use crate::core::common;
use crate::core::common::rect::Rect;
use crate::core::env::Env_Info;
use crate::core::input;
use crate::core::time;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::components::gfx::{C_Animated_Sprite, C_Camera2D, C_Renderable};
use crate::ecs::components::transform::C_Transform2D;
use crate::ecs::entity_manager::{Entity, Entity_Manager};
use crate::game;
use crate::gfx;
use cgmath::Deg;
use std::cell::Ref;
use std::sync::mpsc::Sender;
use std::time::Duration;

pub struct Gameplay_System {
    entity_manager: Entity_Manager,
    entities: Vec<Entity>,
    entity_transform_tx: Option<Sender<(Entity, C_Transform2D)>>,
    camera: Entity,
    camera_transform_tx: Option<Sender<C_Camera2D>>,
}

impl Gameplay_System {
    pub fn new() -> Gameplay_System {
        Gameplay_System {
            entity_manager: Entity_Manager::new(),
            entities: vec![],
            entity_transform_tx: None,
            camera: Entity::INVALID,
            camera_transform_tx: None,
        }
    }

    pub fn init(
        &mut self,
        cfg: &cfg::Config,
        entity_transform_tx: Sender<(Entity, C_Transform2D)>,
        camera_transform_tx: Sender<C_Camera2D>,
    ) -> common::Maybe_Error {
        self.register_all_components();

        self.entity_transform_tx = Some(entity_transform_tx);
        self.camera_transform_tx = Some(camera_transform_tx);
        self.init_demo_entities(cfg);
        //self.init_demo_sprites(cfg);

        Ok(())
    }

    pub fn update(&mut self, dt: &Duration, actions: &input::Action_List) {
        ///// Update all game systems /////
        gfx::animation_system::update(&dt, &mut self.entity_manager);
        game::controllable_system::update(&dt, actions, &mut self.entity_manager);
        self.apply_camera_translation();

        self.update_demo_entites(&dt);
    }

    pub fn get_renderable_entities(&self) -> Vec<(Ref<'_, C_Renderable>, Ref<'_, C_Spatial2D>)> {
        self.entity_manager
            .get_component_tuple::<C_Renderable, C_Spatial2D>()
            .collect()
    }

    fn register_all_components(&mut self) {
        let em = &mut self.entity_manager;

        em.register_component::<C_Spatial2D>();
        em.register_component::<C_Transform2D>();
        em.register_component::<C_Camera2D>();
        em.register_component::<C_Renderable>();
        em.register_component::<C_Animated_Sprite>();
        em.register_component::<C_Controllable>();
    }

    fn apply_camera_translation(&mut self) {
        let delta = self
            .entity_manager
            .get_component::<C_Controllable>(self.camera)
            .unwrap()
            .translation_this_frame;
        let mut camera = self
            .entity_manager
            .get_component_mut::<C_Camera2D>(self.camera)
            .unwrap();
        camera.transform.translate_v(delta);
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

    fn init_demo_entities(&mut self, cfg: &cfg::Config) {
        // #DEMO
        let em = &mut self.entity_manager;
        let etx = self.entity_transform_tx.as_mut().unwrap();

        self.camera = em.new_entity();
        em.add_component::<C_Camera2D>(self.camera);
        {
            let mut ctrl = em.add_component::<C_Controllable>(self.camera);
            ctrl.speed = cfg.get_var_float_or("gameplay/player/player_speed", 300.0);
        }

        let n = 30;
        for i in 0..5000 {
            let entity = em.new_entity();
            let mut t = em.add_component::<C_Transform2D>(entity);
            t.set_origin(50.0, 50.0);
            t.set_position(n as f32 * (i % n) as f32, n as f32 * (i / n) as f32);
            etx.send((entity, *t)).unwrap();
            self.entities.push(entity);
        }
    }

    fn update_demo_entites(&mut self, dt: &Duration) {
        // #DEMO
        let em = &mut self.entity_manager;
        let dt_secs = time::to_secs_frac(dt);
        let etx = self.entity_transform_tx.as_mut().unwrap();
        let ctx = self.camera_transform_tx.as_mut().unwrap();

        ctx.send(*em.get_component::<C_Camera2D>(self.camera).unwrap())
            .unwrap();

        for (i, &e) in self.entities.iter().enumerate() {
            let speed = i as f32 * 0.1;
            if let Some(mut t) = em.get_component_mut::<C_Transform2D>(e) {
                t.rotate(Deg(dt_secs * speed));
                etx.send((e, *t)).unwrap();
            }
        }
    }
}
