use super::controllable_system::C_Controllable;
use crate::controllable_system;
use crate::ecs::components::gfx::{C_Animated_Sprite, C_Camera2D, C_Renderable};
use crate::gfx;
use cgmath::{Deg, InnerSpace};
use ecs_engine::cfg::{self, Cfg_Var};
use ecs_engine::collisions::collider;
use ecs_engine::core::common;
use ecs_engine::core::common::rand;
use ecs_engine::core::common::rect::Rect;
use ecs_engine::core::common::stringid::String_Id;
use ecs_engine::core::common::transform::Transform2D;
use ecs_engine::core::common::vector::Vec2f;
use ecs_engine::core::env::Env_Info;
use ecs_engine::core::scene_tree;
use ecs_engine::core::time;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::ecs_world::{Ecs_World, Entity};
use ecs_engine::ecs::entity_stream::{new_entity_stream, Entity_Stream};
use ecs_engine::gfx as ngfx;
use ecs_engine::input::axes::Virtual_Axes;
use ecs_engine::input::input_system::Game_Action;
use ecs_engine::prelude::*;
use ecs_engine::resources::gfx::{tex_path, Gfx_Resources};
use std::time::Duration;

pub struct Gameplay_System {
    pub ecs_world: Ecs_World,
    entities: Vec<Entity>,
    camera: Entity,
    latest_frame_actions: Vec<Game_Action>,
    latest_frame_axes: Virtual_Axes,
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
        }
    }

    pub fn init(
        &mut self,
        gres: &mut Gfx_Resources,
        env: &Env_Info,
        rng: &mut rand::Default_Rng,
        cfg: &cfg::Config,
    ) -> common::Maybe_Error {
        self.register_all_components();

        self.init_demo_entities(gres, env, rng, cfg);

        Ok(())
    }

    pub fn update(
        &mut self,
        dt: &Duration,
        actions: &[Game_Action],
        axes: &Virtual_Axes,
        cfg: &cfg::Config,
        tracer: Debug_Tracer,
    ) {
        trace!("gameplay_system::update", tracer);
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
        controllable_system::update(&dt, actions, axes, &mut self.ecs_world, cfg);

        #[cfg(any(feature = "prof_scene_tree", feature = "prof_entities_update"))]
        use std::time::Instant;

        #[cfg(feature = "prof_scene_tree")]
        let mut now = Instant::now();

        {
            trace!("scene_tree::copy_transforms", tracer);
            for e in self.entities.iter().copied() {
                if let Some(t) = self.ecs_world.get_component::<C_Spatial2D>(e) {
                    self.scene_tree.set_local_transform(e, &t.local_transform);
                }
            }
        }

        #[cfg(feature = "prof_scene_tree")]
        {
            println!(
                "[prof_scene_tree] copying took {:?} ms",
                now.elapsed().as_micros() as f32 * 0.001
            );
            now = Instant::now();
        }

        {
            trace!("scene_tree::compute_global_transforms", tracer);
            self.scene_tree.compute_global_transforms();
        }

        #[cfg(feature = "prof_scene_tree")]
        {
            println!(
                "[prof_scene_tree] computing took {:.3} ms",
                now.elapsed().as_micros() as f32 * 0.001
            );
            now = Instant::now();
        }

        {
            trace!("scene_tree::backcopy_transforms", tracer);
            for e in self.entities.iter().copied() {
                if let Some(t) = self.ecs_world.get_component_mut::<C_Spatial2D>(e) {
                    t.global_transform = *self.scene_tree.get_global_transform(e).unwrap();
                }
            }
        }

        #[cfg(feature = "prof_scene_tree")]
        println!(
            "[prof_scene_tree] backcopying took {:.3} ms",
            now.elapsed().as_micros() as f32 * 0.001
        );

        #[cfg(feature = "prof_entities_update")]
        let now = Instant::now();

        self.update_demo_entites(&dt);

        #[cfg(feature = "prof_entities_update")]
        println!(
            "[prof_entities_update] update took {:.3} ms",
            now.elapsed().as_micros() as f32 * 0.001
        );
    }

    pub fn realtime_update(
        &mut self,
        real_dt: &Duration,
        actions: &[Game_Action],
        axes: &Virtual_Axes,
        cfg: &cfg::Config,
        tracer: Debug_Tracer,
    ) {
        trace!("gameplay_system::realtime_update", tracer);
        self.update_camera(real_dt, actions, axes, cfg);
    }

    #[cfg(debug_assertions)]
    pub fn step(&mut self, dt: &Duration, cfg: &cfg::Config, tracer: Debug_Tracer) {
        self.update_with_latest_frame_actions(dt, cfg, tracer);
    }

    #[cfg(debug_assertions)]
    pub fn print_debug_info(&self) {
        //self.ecs_world.print_debug_info();
    }

    fn update_with_latest_frame_actions(
        &mut self,
        dt: &Duration,
        cfg: &cfg::Config,
        tracer: Debug_Tracer,
    ) {
        let mut actions = vec![];
        std::mem::swap(&mut self.latest_frame_actions, &mut actions);
        let mut axes = Virtual_Axes::default();
        std::mem::swap(&mut self.latest_frame_axes, &mut axes);
        self.update(&dt, &actions, &axes, cfg, tracer);
    }

    pub fn get_renderable_entities(&self) -> Entity_Stream {
        new_entity_stream(&self.ecs_world)
            .require::<C_Renderable>()
            .require::<C_Spatial2D>()
            .build()
    }

    pub fn get_camera(&self) -> C_Camera2D {
        *self
            .ecs_world
            .get_components::<C_Camera2D>()
            .first()
            .unwrap()
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
        _actions: &[Game_Action],
        axes: &Virtual_Axes,
        cfg: &cfg::Config,
    ) {
        // @Incomplete
        let movement = get_movement_from_input(axes);
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

        self.apply_camera_translation(v);
    }

    fn apply_camera_translation(&mut self, delta: Vec2f) {
        let camera = self
            .ecs_world
            .get_component_mut::<C_Camera2D>(self.camera)
            .unwrap();
        camera.transform.translate_v(delta);
    }

    fn init_demo_entities(
        &mut self,
        rsrc: &mut Gfx_Resources,
        env: &Env_Info,
        rng: &mut rand::Default_Rng,
        cfg: &cfg::Config,
    ) {
        // #DEMO
        let em = &mut self.ecs_world;

        self.camera = em.new_entity();
        {
            let cam = em.add_component::<C_Camera2D>(self.camera);
            cam.transform.set_scale(2.5, 2.5);
        }

        {
            let mut ctrl = em.add_component::<C_Controllable>(self.camera);
            ctrl.speed = Cfg_Var::new("gameplay/player/player_speed", cfg);
        }

        let mut prev_entity: Option<Entity> = None;
        let mut fst_entity: Option<Entity> = None;
        let n_frames = 4;
        for i in 0..1000 {
            let entity = em.new_entity();
            let (sw, sh) = {
                let mut rend = em.add_component::<C_Renderable>(entity);
                rend.texture = rsrc.load_texture(&tex_path(&env, "plant.png"));
                assert!(rend.texture.is_some(), "Could not load plant texture!");
                let (sw, sh) = ngfx::render::get_texture_size(rsrc.get_texture(rend.texture));
                rend.rect = Rect::new(0, 0, sw as i32 / (n_frames as i32), sh as i32);
                (sw, sh)
            };
            {
                let t = em.add_component::<C_Spatial2D>(entity);
                let x = rand::rand_01(rng);
                let y = rand::rand_01(rng);
                t.local_transform
                    .set_origin((sw / n_frames) as f32 * 0.5, (sh / n_frames) as f32 * 0.5);
                if i > 0 {
                    t.local_transform.set_position(x * 4248.0, y * 4248.0);
                }
                self.scene_tree.add(entity, fst_entity, &t.local_transform);
            }
            {
                let c = em.add_component::<collider::Collider>(entity);
                c.shape = collider::Collider_Shape::Rect {
                    width: (sw / n_frames) as f32,
                    height: sh as f32,
                };
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

    fn update_demo_entites(&mut self, dt: &Duration) {
        // #DEMO
        let em = &mut self.ecs_world;
        let dt_secs = time::to_secs_frac(dt);

        let mut stream = new_entity_stream(em)
            .require::<C_Controllable>()
            .require::<C_Spatial2D>()
            .build();
        loop {
            let entity = stream.next(em);
            if entity.is_none() {
                break;
            }
            let entity = entity.unwrap();
            let ctrl = em.get_component::<C_Controllable>(entity).unwrap();
            let transl = ctrl.translation_this_frame;
            let spat = em.get_component_mut::<C_Spatial2D>(entity).unwrap();
            spat.local_transform.translate_v(transl);
            spat.velocity.x = transl.x;
            spat.velocity.y = transl.y;
        }

        for (i, t) in em
            .get_components_mut::<C_Spatial2D>()
            .iter_mut()
            .enumerate()
        {
            let speed = 1.0;
            if i % 10 == 1 {
                t.local_transform.rotate(Deg(dt_secs * speed));
            }
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
