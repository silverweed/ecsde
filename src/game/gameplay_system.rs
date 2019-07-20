use super::controllable_system::C_Controllable;
use crate::cfg;
use crate::core::common;
use crate::core::common::rect::Rect;
use crate::core::common::vector::Vec2f;
use crate::core::env::Env_Info;
use crate::core::input;
use crate::core::msg;
use crate::core::time;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::components::gfx::{C_Animated_Sprite, C_Camera2D, C_Renderable};
use crate::ecs::components::transform::C_Transform2D;
use crate::ecs::entity_manager::{Entity, Entity_Manager};
use crate::game;
use crate::gfx;
use crate::resources::gfx::{tex_path, Gfx_Resources};
use cgmath::Deg;
use std::cell::Ref;
use std::time::Duration;

pub struct Gameplay_System {
    entity_manager: Entity_Manager,
    entities: Vec<Entity>,
    camera: Entity,
    latest_frame_actions: input::Action_List,
}

pub enum Gameplay_System_Msg {
    Step(Duration),
    Print_Entity_Manager_Debug_Info,
}

impl msg::Msg_Responder for Gameplay_System {
    type Msg_Data = Gameplay_System_Msg;
    type Resp_Data = ();

    fn send_message(&mut self, msg: Gameplay_System_Msg) -> () {
        match msg {
            Gameplay_System_Msg::Step(dt) => self.update_with_latest_frame_actions(&dt),
            Gameplay_System_Msg::Print_Entity_Manager_Debug_Info => {
                #[cfg(debug_assertions)]
                self.entity_manager.print_debug_info()
            }
        }
        ()
    }
}

impl Gameplay_System {
    pub fn new() -> Gameplay_System {
        Gameplay_System {
            entity_manager: Entity_Manager::new(),
            entities: vec![],
            camera: Entity::INVALID,
            latest_frame_actions: input::Action_List::default(),
        }
    }

    pub fn init(
        &mut self,
        gres: &mut Gfx_Resources,
        env: &Env_Info,
        cfg: &cfg::Config,
    ) -> common::Maybe_Error {
        self.register_all_components();

        self.init_demo_entities(gres, env, cfg);
        //self.init_demo_sprites(cfg);

        Ok(())
    }

    pub fn update(&mut self, dt: &Duration, actions: &input::Action_List) {
        // Used for stepping
        self.latest_frame_actions = actions.clone();

        ///// Update all game systems /////
        gfx::animation_system::update(&dt, &mut self.entity_manager);
        game::controllable_system::update(&dt, actions, &mut self.entity_manager);

        self.update_demo_entites(&dt);
    }

    pub fn realtime_update(&mut self, real_dt: &Duration, actions: &input::Action_List) {
        self.update_camera(real_dt, actions);
    }

    fn update_with_latest_frame_actions(&mut self, dt: &Duration) {
        let mut actions = input::Action_List::default();
        std::mem::swap(&mut self.latest_frame_actions, &mut actions);
        self.update(&dt, &actions);
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
        em.register_component::<C_Transform2D>();
        em.register_component::<C_Camera2D>();
        em.register_component::<C_Renderable>();
        em.register_component::<C_Animated_Sprite>();
        em.register_component::<C_Controllable>();
    }

    fn update_camera(&mut self, real_dt: &Duration, actions: &input::Action_List) {
        let movement = input::get_normalized_movement_from_input(actions);
        let camera_ctrl = self
            .entity_manager
            .get_component_mut::<C_Controllable>(self.camera);
        if camera_ctrl.is_none() {
            return;
        }

        let v = {
            let real_dt_secs = time::to_secs_frac(real_dt);
            let mut camera_ctrl = camera_ctrl.unwrap();
            let speed = *camera_ctrl.speed;
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

    fn init_demo_entities(&mut self, rsrc: &mut Gfx_Resources, env: &Env_Info, cfg: &cfg::Config) {
        // #DEMO
        let em = &mut self.entity_manager;

        self.camera = em.new_entity();
        em.add_component::<C_Camera2D>(self.camera);
        {
            let mut ctrl = em.add_component::<C_Controllable>(self.camera);
            ctrl.speed = cfg.get_var_float_or("gameplay/player/player_speed", 300.0);
        }

        let n = 30;
        for i in 0..200 {
            let entity = em.new_entity();
            let (sw, sh) = {
                let mut rend = em.add_component::<C_Renderable>(entity);
                rend.texture = rsrc.load_texture(&tex_path(&env, "yv.png"));
                assert!(rend.texture.is_some(), "Could not load yv texture!");
                let (sw, sh) = gfx::render::get_texture_size(rsrc.get_texture(rend.texture));
                rend.rect = Rect::new(0, 0, sw, sh);
                (sw, sh)
            };
            {
                let mut t = em.add_component::<C_Spatial2D>(entity);
                t.transform.set_origin(sw as f32 * 0.5, sh as f32 * 0.5);
                t.transform
                    .set_position(n as f32 * (i % n) as f32, n as f32 * (i / n) as f32);
            }
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
            spat.transform.translate_v(transl);
            spat.velocity.x = transl.x;
            spat.velocity.y = transl.y;
        }

        for (i, t) in em
            .get_components_mut::<C_Spatial2D>()
            .iter_mut()
            .enumerate()
        {
            let speed = i as f32 * 2.1;
            t.transform.rotate(Deg(dt_secs * speed));
        }
    }
}
