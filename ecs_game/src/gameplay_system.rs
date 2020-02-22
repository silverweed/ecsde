use super::controllable_system::C_Controllable;
use crate::controllable_system;
use crate::input_utils::{get_movement_from_input, Input_Config};
use crate::movement_system;
use ecs_engine::cfg::{self, Cfg_Var};
use ecs_engine::collisions::collider;
use ecs_engine::common;
use ecs_engine::common::colors;
use ecs_engine::common::rect::Rect;
use ecs_engine::common::stringid::String_Id;
use ecs_engine::common::transform::Transform2D;
use ecs_engine::common::vector::Vec2f;
use ecs_engine::core::env::Env_Info;
use ecs_engine::core::rand;
use ecs_engine::core::scene_tree;
use ecs_engine::core::time;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::components::gfx::{C_Animated_Sprite, C_Camera2D, C_Renderable};
use ecs_engine::ecs::ecs_world::{Ecs_World, Entity};
use ecs_engine::ecs::entity_stream::new_entity_stream;
use ecs_engine::gfx;
use ecs_engine::gfx as ngfx;
use ecs_engine::input::axes::Virtual_Axes;
use ecs_engine::input::bindings::keyboard;
use ecs_engine::input::input_system::{Action_Kind, Game_Action};
use ecs_engine::prelude::*;
use ecs_engine::resources::gfx::{tex_path, Gfx_Resources};
use std::time::Duration;

pub struct Gameplay_System_Config {
    pub n_entities_to_spawn: usize,
}

pub struct Gameplay_System {
    pub ecs_world: Ecs_World,
    entities: Vec<Entity>,
    camera: Entity,
    latest_frame_actions: Vec<Game_Action>,
    latest_frame_axes: Virtual_Axes,
    pub input_cfg: Input_Config,
    scene_tree: scene_tree::Scene_Tree,
}

impl Gameplay_System {
    pub fn new() -> Gameplay_System {
        Gameplay_System {
            ecs_world: Ecs_World::new(),
            entities: vec![],
            camera: Entity::INVALID,
            latest_frame_actions: vec![],
            latest_frame_axes: Virtual_Axes::default(),
            scene_tree: scene_tree::Scene_Tree::new(),
            input_cfg: Input_Config::default(),
        }
    }

    pub fn init(
        &mut self,
        gres: &mut Gfx_Resources,
        env: &Env_Info,
        rng: &mut rand::Default_Rng,
        cfg: &cfg::Config,
        gs_cfg: Gameplay_System_Config,
    ) -> common::Maybe_Error {
        self.input_cfg = read_input_cfg(cfg);
        self.register_all_components();
        self.init_demo_entities(gres, env, rng, cfg, gs_cfg);

        Ok(())
    }

    pub fn update(
        &mut self,
        dt: &Duration,
        time: &time::Time,
        actions: &[Game_Action],
        axes: &Virtual_Axes,
        cfg: &cfg::Config,
        _tracer: Debug_Tracer,
    ) {
        trace!("gameplay_system::update", _tracer);
        // Used for stepping
        self.latest_frame_actions = actions.to_vec();

        ///// Update all game systems /////
        {
            let stream = new_entity_stream(&self.ecs_world)
                .require::<C_Renderable>()
                .require::<C_Animated_Sprite>()
                .build();
            gfx::animation_system::update(&dt, &mut self.ecs_world, stream);
        }
        controllable_system::update(&dt, actions, axes, &mut self.ecs_world, self.input_cfg, cfg);

        {
            trace!("scene_tree::copy_transforms", _tracer);
            for e in self.entities.iter().copied() {
                if let Some(t) = self.ecs_world.get_component::<C_Spatial2D>(e) {
                    self.scene_tree.set_local_transform(e, &t.local_transform);
                }
            }
        }

        {
            trace!("scene_tree::compute_global_transforms", _tracer);
            self.scene_tree.compute_global_transforms();
        }

        {
            trace!("scene_tree::backcopy_transforms", _tracer);
            for e in self.entities.iter().copied() {
                if let Some(t) = self.ecs_world.get_component_mut::<C_Spatial2D>(e) {
                    t.global_transform = *self.scene_tree.get_global_transform(e).unwrap();
                }
            }
        }

        self.update_demo_entites(&dt, time);

        movement_system::update(&dt, &mut self.ecs_world);
    }

    pub fn realtime_update(
        &mut self,
        real_dt: &Duration,
        _time: &time::Time,
        actions: &[Game_Action],
        axes: &Virtual_Axes,
        cfg: &cfg::Config,
        _tracer: Debug_Tracer,
    ) {
        trace!("gameplay_system::realtime_update", _tracer);
        self.update_camera(real_dt, actions, axes, cfg);
    }

    #[cfg(debug_assertions)]
    pub fn step(
        &mut self,
        dt: &Duration,
        time: &time::Time,
        cfg: &cfg::Config,
        _tracer: Debug_Tracer,
    ) {
        self.update_with_latest_frame_actions(dt, time, cfg, _tracer);
    }

    #[cfg(debug_assertions)]
    pub fn print_debug_info(&self) {
        // @Incomplete
        //self.ecs_world.print_debug_info();
    }

    fn update_with_latest_frame_actions(
        &mut self,
        dt: &Duration,
        time: &time::Time,
        cfg: &cfg::Config,
        _tracer: Debug_Tracer,
    ) {
        let mut actions = vec![];
        std::mem::swap(&mut self.latest_frame_actions, &mut actions);
        let mut axes = Virtual_Axes::default();
        std::mem::swap(&mut self.latest_frame_axes, &mut axes);
        self.update(&dt, time, &actions, &axes, cfg, _tracer);
    }

    pub fn get_camera(&self) -> &C_Camera2D {
        self.ecs_world
            .get_components::<C_Camera2D>()
            .first()
            .unwrap()
    }

    pub fn move_camera_to(&mut self, pos: Vec2f) {
        self.ecs_world
            .get_component_mut::<C_Camera2D>(self.camera)
            .unwrap()
            .transform
            .set_position_v(pos);
    }

    fn register_all_components(&mut self) {
        let em = &mut self.ecs_world;

        em.register_component::<C_Spatial2D>();
        em.register_component::<Transform2D>();
        em.register_component::<C_Camera2D>();
        em.register_component::<C_Renderable>();
        em.register_component::<C_Animated_Sprite>();
        em.register_component::<C_Controllable>();
        em.register_component::<collider::Collider>();
    }

    fn update_camera(
        &mut self,
        real_dt: &Duration,
        actions: &[Game_Action],
        axes: &Virtual_Axes,
        cfg: &cfg::Config,
    ) {
        // @Incomplete
        let movement = get_movement_from_input(axes, self.input_cfg, cfg);
        let camera_ctrl = self
            .ecs_world
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

        let camera = self
            .ecs_world
            .get_component_mut::<C_Camera2D>(self.camera)
            .unwrap();

        let sx = camera.transform.scale().x;
        let v = v * sx;

        let mut add_scale = Vec2f::new(0., 0.);
        const BASE_CAM_DELTA_ZOOM_PER_SCROLL: f32 = 0.2;

        if keyboard::is_key_pressed(keyboard::Key::LControl) {
            for action in actions {
                match action {
                    (name, Action_Kind::Pressed) if *name == String_Id::from("camera_zoom_up") => {
                        add_scale.x -= BASE_CAM_DELTA_ZOOM_PER_SCROLL * sx;
                        add_scale.y = add_scale.x;
                    }
                    (name, Action_Kind::Pressed)
                        if *name == String_Id::from("camera_zoom_down") =>
                    {
                        add_scale.x += BASE_CAM_DELTA_ZOOM_PER_SCROLL * sx;
                        add_scale.y = add_scale.x;
                    }
                    _ => (),
                }
            }
        }

        camera.transform.translate_v(v);
        camera.transform.add_scale_v(add_scale);

        // DEBUG: center camera on player
        {
            let mut stream = new_entity_stream(&self.ecs_world)
                .require::<C_Controllable>()
                .exclude::<C_Camera2D>()
                .build();
            let moved = stream.next(&self.ecs_world);
            if let Some(moved) = moved {
                let pos = self
                    .ecs_world
                    .get_component::<C_Spatial2D>(moved)
                    .unwrap()
                    .global_transform
                    .position();
                let camera = self
                    .ecs_world
                    .get_component_mut::<C_Camera2D>(self.camera)
                    .unwrap();
                camera
                    .transform
                    .set_position_v(pos + Vec2f::new(-300., -300.));
            }
        }
    }

    fn init_demo_entities(
        &mut self,
        rsrc: &mut Gfx_Resources,
        env: &Env_Info,
        rng: &mut rand::Default_Rng,
        cfg: &cfg::Config,
        gs_cfg: Gameplay_System_Config,
    ) {
        // #DEMO
        let em = &mut self.ecs_world;

        self.camera = em.new_entity();
        {
            let cam = em.add_component::<C_Camera2D>(self.camera);
            //cam.transform.set_scale(2.5, 2.5);
            cam.transform.set_position(-300., -300.);
        }

        {
            let mut ctrl = em.add_component::<C_Controllable>(self.camera);
            ctrl.speed = Cfg_Var::new("game/gameplay/player/player_speed", cfg);
        }

        let mut prev_entity: Option<Entity> = None;
        let mut fst_entity: Option<Entity> = None;
        let n_frames = 4;
        for i in 0..gs_cfg.n_entities_to_spawn {
            let entity = em.new_entity();
            let (sw, sh) = {
                let mut rend = em.add_component::<C_Renderable>(entity);
                //rend.texture = rsrc.load_texture(&tex_path(&env, "yv.png"));
                rend.texture = rsrc.load_texture(&tex_path(&env, "plant.png"));
                assert!(rend.texture.is_some(), "Could not load texture!");
                rend.modulate = if i == 1 {
                    colors::rgb(0, 255, 0)
                } else {
                    colors::WHITE
                };
                let (sw, sh) = ngfx::render::get_texture_size(rsrc.get_texture(rend.texture));
                rend.rect = Rect::new(0, 0, sw as i32 / (n_frames as i32), sh as i32);
                (sw, sh)
            };
            if i == 1 {
                let ctr = em.add_component::<C_Controllable>(entity);
                ctr.speed = Cfg_Var::new("game/gameplay/player/player_speed", cfg);
            }
            {
                let t = em.add_component::<C_Spatial2D>(entity);
                let x = rand::rand_01(rng);
                let y = rand::rand_01(rng);
                if i > 0 {
                    //t.local_transform.set_position(i as f32 * 242.0, 0.);
                    t.local_transform.set_position(x * 500., 1. * y * 1500.);
                }
                self.scene_tree.add(entity, fst_entity, &t.local_transform);
            }
            {
                let c = em.add_component::<collider::Collider>(entity);
                let width = (sw / n_frames) as f32;
                let height = sh as f32;
                c.shape = collider::Collider_Shape::Rect { width, height };
                //c.shape = collider::Collider_Shape::Circle {
                //radius: width.max(height) * 0.5,
                //};
                c.offset = -Vec2f::new(width * 0.5, height * 0.5);
            }
            {
                let s = em.add_component::<C_Animated_Sprite>(entity);
                s.n_frames = n_frames;
                s.frame_time = 0.16;
            }
            prev_entity = Some(entity);
            if fst_entity.is_none() {
                fst_entity = Some(entity);
            }
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

    fn update_demo_entites(&mut self, dt: &Duration, time: &time::Time) {
        // #DEMO
        let em = &mut self.ecs_world;
        let dt_secs = time::to_secs_frac(dt);

        //let mut stream = new_entity_stream(em)
        //.require::<C_Controllable>()
        //.require::<C_Spatial2D>()
        //.build();
        //loop {
        //let entity = stream.next(em);
        //if entity.is_none() {
        //break;
        //}
        //let entity = entity.unwrap();
        //let ctrl = em.get_component::<C_Controllable>(entity).unwrap();
        //let transl = ctrl.translation_this_frame;
        //let spat = em.get_component_mut::<C_Spatial2D>(entity).unwrap();
        //spat.local_transform.translate_v(transl);
        //spat.velocity.x = transl.x;
        //spat.velocity.y = transl.y;
        //}

        let mut stream = new_entity_stream(em)
            .require::<C_Spatial2D>()
            .exclude::<C_Controllable>()
            .build();
        let mut i = 0;
        loop {
            let entity = stream.next(em);
            if entity.is_none() {
                break;
            }
            let entity = entity.unwrap();
            let t = em.get_component_mut::<C_Spatial2D>(entity).unwrap();

            if i == 1 {
                t.velocity = Vec2f::new(-50.0, 0.);
            }
            {
                //use ecs_engine::common::angle::deg;
                //let speed = 90.0;
                //if i % 10 == 0 {
                //t.local_transform.rotate(deg(dt_secs * speed));
                //}
                //let prev_pos = t.local_transform.position();
                //t.local_transform.set_position(
                //(time::to_secs_frac(&time.get_game_time()) + i as f32 * 0.4).sin() * 100.,
                //3.,
                //);
                //t.velocity = t.local_transform.position() - prev_pos;
                //t.local_transform.set_rotation(deg(30.));
            }

            i += 1;
        }
    }
}

fn read_input_cfg(cfg: &cfg::Config) -> Input_Config {
    Input_Config {
        joy_deadzone: Cfg_Var::new("game/input/joystick/deadzone", cfg),
    }
}
