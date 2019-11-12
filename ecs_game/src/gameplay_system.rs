use super::controllable_system::C_Controllable;
use crate::controllable_system;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::components::gfx::{C_Animated_Sprite, C_Camera2D, C_Renderable};
use crate::ecs::entity_manager::{Entity, Entity_Manager};
use crate::gfx;
use crate::scene_tree;
use cgmath::{Deg, InnerSpace};
use ecs_engine::cfg::{self, Cfg_Var};
use ecs_engine::core::common;
use ecs_engine::core::common::rect::Rect;
use ecs_engine::core::common::stringid::String_Id;
use ecs_engine::core::common::transform::Transform2D;
use ecs_engine::core::common::vector::Vec2f;
use ecs_engine::core::env::Env_Info;
use ecs_engine::core::time;
use ecs_engine::gfx as ngfx;
use ecs_engine::input::axes::Virtual_Axes;
use ecs_engine::input::input_system::Game_Action;
use ecs_engine::resources::gfx::{tex_path, Gfx_Resources};
use std::cell::Ref;
use std::time::Duration;

pub struct Gameplay_System {
    entity_manager: Entity_Manager,
    entities: Vec<Entity>,
    camera: Entity,
    latest_frame_actions: Vec<Game_Action>,
    latest_frame_axes: Virtual_Axes,
    scene_tree: scene_tree::Scene_Tree,
}

impl Gameplay_System {
    pub fn new() -> Gameplay_System {
        Gameplay_System {
            entity_manager: Entity_Manager::new(),
            entities: vec![],
            camera: Entity::INVALID,
            latest_frame_actions: vec![],
            latest_frame_axes: Virtual_Axes::default(),
            scene_tree: scene_tree::Scene_Tree::new(),
        }
    }

    pub fn init(&mut self, gres: &mut Gfx_Resources, env: &Env_Info) -> common::Maybe_Error {
        self.register_all_components();

        self.init_demo_entities(gres, env);
        //self.init_demo_sprites(cfg);

        Ok(())
    }

    pub fn update(
        &mut self,
        dt: &Duration,
        actions: &[Game_Action],
        axes: &Virtual_Axes,
        cfg: &cfg::Config,
    ) {
        // Used for stepping
        self.latest_frame_actions = actions.to_vec();

        ///// Update all game systems /////
        gfx::animation_system::update(&dt, &mut self.entity_manager);
        controllable_system::update(&dt, actions, axes, &mut self.entity_manager, cfg);

        for e in &self.entities {
            if let Some(t) = self.entity_manager.get_component::<C_Spatial2D>(*e) {
                self.scene_tree.set_local_transform(*e, &t.local_transform);
            }
        }
        self.scene_tree.compute_global_transforms();
        for e in &self.entities {
            if let Some(mut t) = self.entity_manager.get_component_mut::<C_Spatial2D>(*e) {
                t.global_transform = *self.scene_tree.get_global_transform(*e).unwrap();
            }
        }

        self.update_demo_entites(&dt);
    }

    pub fn realtime_update(
        &mut self,
        real_dt: &Duration,
        actions: &[Game_Action],
        axes: &Virtual_Axes,
        cfg: &cfg::Config,
    ) {
        self.update_camera(real_dt, actions, axes, cfg);
    }

    #[cfg(debug_assertions)]
    pub fn step(&mut self, dt: &Duration, cfg: &cfg::Config) {
        self.update_with_latest_frame_actions(dt, cfg);
    }

    #[cfg(debug_assertions)]
    pub fn print_debug_info(&self) {
        self.entity_manager.print_debug_info();
    }

    fn update_with_latest_frame_actions(&mut self, dt: &Duration, cfg: &cfg::Config) {
        let mut actions = vec![];
        std::mem::swap(&mut self.latest_frame_actions, &mut actions);
        let mut axes = Virtual_Axes::default();
        std::mem::swap(&mut self.latest_frame_axes, &mut axes);
        self.update(&dt, &actions, &axes, cfg);
    }

    pub fn get_renderable_entities(&self) -> Vec<(Ref<'_, C_Renderable>, Ref<'_, C_Spatial2D>)> {
        self.entity_manager
            .get_component_tuple::<C_Renderable, C_Spatial2D>()
            .collect()
    }

    pub fn get_camera(&self) -> C_Camera2D {
        **self
            .entity_manager
            .get_components::<C_Camera2D>()
            .first()
            .unwrap()
    }

    fn register_all_components(&mut self) {
        let em = &mut self.entity_manager;

        em.register_component::<C_Spatial2D>();
        em.register_component::<Transform2D>();
        em.register_component::<C_Camera2D>();
        em.register_component::<C_Renderable>();
        em.register_component::<C_Animated_Sprite>();
        em.register_component::<C_Controllable>();
    }

    fn update_camera(
        &mut self,
        real_dt: &Duration,
        _actions: &[Game_Action],
        axes: &Virtual_Axes,
        cfg: &cfg::Config,
    ) {
        // @Incomplete
        let movement = get_movement_from_input(axes);
        let camera_ctrl = self
            .entity_manager
            .get_component_mut::<C_Controllable>(self.camera);
        if camera_ctrl.is_none() {
            return;
        }

        let v = {
            let real_dt_secs = time::to_secs_frac(real_dt);
            let mut camera_ctrl = camera_ctrl.unwrap();
            let speed = camera_ctrl.speed.read(cfg);
            let velocity = movement * speed;
            let v = velocity * real_dt_secs;
            camera_ctrl.translation_this_frame = v;
            v
        };

        self.apply_camera_translation(v);
    }

    fn apply_camera_translation(&mut self, delta: Vec2f) {
        let mut camera = self
            .entity_manager
            .get_component_mut::<C_Camera2D>(self.camera)
            .unwrap();
        camera.transform.translate_v(delta);
    }

    // #DEMO
    fn init_demo_sprites(&mut self) {
        let em = &mut self.entity_manager;
        let yv = em.new_entity();
        self.entities.push(yv);
        {
            let mut s = em.add_component::<C_Spatial2D>(yv);
            s.local_transform.set_position(300.0, 200.0);
            s.local_transform.set_scale(3.0, 3.0);
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
            s.local_transform.set_position(400.0, 500.0);
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
            ctrl.speed = Cfg_Var::new("gameplay/player/player_speed");
        }
    }

    fn init_demo_entities(&mut self, rsrc: &mut Gfx_Resources, env: &Env_Info) {
        // #DEMO
        let em = &mut self.entity_manager;

        self.camera = em.new_entity();
        em.add_component::<C_Camera2D>(self.camera);
        {
            let mut ctrl = em.add_component::<C_Controllable>(self.camera);
            ctrl.speed = Cfg_Var::new("gameplay/player/player_speed");
        }

        let mut prev_entity: Option<Entity> = None;
        for i in 0..10 {
            let entity = em.new_entity();
            let (sw, sh) = {
                let mut rend = em.add_component::<C_Renderable>(entity);
                rend.texture = rsrc.load_texture(&tex_path(&env, "yv.png"));
                assert!(rend.texture.is_some(), "Could not load yv texture!");
                let (sw, sh) = ngfx::render::get_texture_size(rsrc.get_texture(rend.texture));
                rend.rect = Rect::new(0, 0, sw as i32, sh as i32);
                (sw, sh)
            };
            {
                let mut t = em.add_component::<C_Spatial2D>(entity);
                //t.local_transform.set_origin(sw as f32 * 0.5, sh as f32 * 0.5);
                if i > 0 {
                    t.local_transform.set_position(20.0, 0.0);
                }
                self.scene_tree.add(entity, prev_entity, &t.local_transform);
            }
            prev_entity = Some(entity);
            //{
            //    let mut t = em.add_component::<C_Spatial2D>(entity);
            //    t.transform.set_origin(sw as f32 * 0.5, sh as f32 * 0.5);
            //    t.transform
            //        .set_position(n as f32 * (i % n) as f32, n as f32 * (i / n) as f32);
            //}
            //{
            //let mut ctrl = em.add_component::<C_Controllable>(entity);
            //ctrl.speed = cfg.get_var_float_or("gameplay/player/player_speed", 300.0);
            //}
            self.entities.push(entity);
        }
    }

    fn update_demo_entites(&mut self, dt: &Duration) {
        // #DEMO
        let em = &mut self.entity_manager;
        let dt_secs = time::to_secs_frac(dt);

        for (ctrl, spat) in em.get_component_tuple_mut::<C_Controllable, C_Spatial2D>() {
            let transl = ctrl.borrow().translation_this_frame;
            let mut spat = spat.borrow_mut();
            spat.local_transform.translate_v(transl);
            spat.velocity.x = transl.x;
            spat.velocity.y = transl.y;
        }

        for (i, t) in em
            .get_components_mut::<C_Spatial2D>()
            .iter_mut()
            .enumerate()
        {
            let speed = 20.0;
            t.local_transform.rotate(Deg(dt_secs * speed));
        }
    }
}

pub fn get_movement_from_input(axes: &Virtual_Axes) -> Vec2f {
    Vec2f::new(
        axes.get_axis_value(String_Id::from("horizontal")),
        axes.get_axis_value(String_Id::from("vertical")),
    )
}

pub fn get_normalized_movement_from_input(axes: &Virtual_Axes) -> Vec2f {
    let m = get_movement_from_input(axes);
    if m.magnitude2() == 0.0 {
        m
    } else {
        m.normalize()
    }
}
