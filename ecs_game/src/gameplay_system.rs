#![allow(warnings)] // @Temporary

use super::systems::controllable_system::{self, C_Controllable};
use super::systems::dumb_movement_system;
use super::systems::ground_collision_calculation_system::Ground_Collision_Calculation_System;
use crate::input_utils::{get_movement_from_input, Input_Config};
use crate::load::load_system;
use crate::movement_system;
use crate::spatial::World_Chunks;
use crate::Game_Resources;
use ecs_engine::cfg::{self, Cfg_Var};
use ecs_engine::collisions::collider;
use ecs_engine::common;
use ecs_engine::common::colors;
use ecs_engine::common::rect::Rect;
use ecs_engine::common::stringid::String_Id;
use ecs_engine::common::transform::Transform2D;
use ecs_engine::common::vector::Vec2f;
use ecs_engine::core::app::Engine_State;
use ecs_engine::core::env::Env_Info;
use ecs_engine::core::rand;
use ecs_engine::core::time;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::components::gfx::{C_Animated_Sprite, C_Camera2D, C_Renderable};
use ecs_engine::ecs::ecs_world::{Ecs_World, Entity};
use ecs_engine::ecs::entity_stream::new_entity_stream;
use ecs_engine::events::evt_register::Event_Register;
use ecs_engine::gfx;
use ecs_engine::gfx::render::batcher::Batches;
use ecs_engine::input::axes::Virtual_Axes;
use ecs_engine::input::bindings::keyboard;
use ecs_engine::input::input_system::{Action_Kind, Game_Action};
use ecs_engine::resources::gfx::{tex_path, Gfx_Resources};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

#[derive(Default, Copy, Clone)]
pub struct Gameplay_System_Config {
    pub n_entities_to_spawn: usize,
}

// A Level is what gets loaded and unloaded
pub struct Level {
    pub id: String_Id,
    pub world: Ecs_World,
    pub chunks: World_Chunks,
    pub cameras: Vec<Entity>,
    pub active_camera: usize, // index inside 'cameras'
}

impl Level {
    // @Temporary: we need to better decide how to handle cameras
    pub fn get_camera(&self) -> &C_Camera2D {
        self.world.get_components::<C_Camera2D>().next().unwrap()
    }

    // @Temporary
    pub fn move_camera_to(&mut self, pos: Vec2f) {
        self.world
            .get_components_mut::<C_Camera2D>()
            .next()
            .unwrap()
            .transform
            .set_position_v(pos);
    }
}

pub struct Gameplay_System {
    pub loaded_levels: Vec<Arc<Mutex<Level>>>,
    pub active_levels: Vec<usize>, // indices inside 'loaded_levels'
    pub input_cfg: Input_Config,
    cfg: Gameplay_System_Config,

    ground_collision_calc_system: Ground_Collision_Calculation_System,

    #[cfg(debug_assertions)]
    debug_data: Debug_Data,
}

#[cfg(debug_assertions)]
#[derive(Default)]
struct Debug_Data {
    pub latest_frame_actions: Vec<Game_Action>,
    pub latest_frame_axes: Virtual_Axes,
}

impl Gameplay_System {
    pub fn new() -> Gameplay_System {
        Gameplay_System {
            loaded_levels: vec![],
            active_levels: vec![],
            input_cfg: Input_Config::default(),
            cfg: Gameplay_System_Config::default(),
            ground_collision_calc_system: Ground_Collision_Calculation_System::new(),
            #[cfg(debug_assertions)]
            debug_data: Debug_Data::default(),
        }
    }

    pub fn first_active_level(&self) -> Option<MutexGuard<Level>> {
        self.active_levels
            .get(0)
            .map(|idx| self.loaded_levels[*idx].lock().unwrap())
    }

    #[inline]
    pub fn foreach_active_level<F: FnMut(&mut Level)>(&self, mut f: F) {
        for &idx in &self.active_levels {
            let mut level = self.loaded_levels[idx]
                .lock()
                .unwrap_or_else(|err| fatal!("Failed to lock level {}: {}", idx, err));
            f(&mut *level);
        }
    }

    pub fn init(
        &mut self,
        gres: &mut Gfx_Resources,
        engine_state: &mut Engine_State,
        gs_cfg: Gameplay_System_Config,
    ) -> common::Maybe_Error {
        self.input_cfg = read_input_cfg(&engine_state.config);
        self.cfg = gs_cfg;
        self.ground_collision_calc_system.init(engine_state);

        Ok(())
    }

    pub fn load_test_level(
        &mut self,
        engine_state: &mut Engine_State,
        game_res: &mut Game_Resources,
        level_batches: &mut HashMap<String_Id, Batches>,
    ) {
        let level_id = String_Id::from("test");
        let mut level = load_system::level_load_sync(level_id, engine_state, game_res, self.cfg);

        level.chunks.init(engine_state);

        self.loaded_levels.push(Arc::new(Mutex::new(level)));
        self.active_levels.push(self.loaded_levels.len() - 1);

        level_batches.insert(level_id, Batches::default());

        #[cfg(debug_assertions)]
        {
            engine_state.debug_systems.new_debug_painter_for_level(
                level_id,
                &mut game_res.gfx,
                &engine_state.env,
            );
        }
    }

    pub fn update(
        &mut self,
        dt: &Duration,
        actions: &[Game_Action],
        axes: &Virtual_Axes,
        cfg: &cfg::Config,
        rng: &mut rand::Default_Rng,
    ) {
        trace!("gameplay_system::update");

        #[cfg(debug_assertions)]
        {
            self.debug_data.latest_frame_actions = actions.to_vec();
        }

        ///// Update all game systems in all worlds /////
        // Note: inlining foreach_active_levels because we don't want to borrow self.
        for &idx in &self.active_levels {
            let mut level = self.loaded_levels[idx]
                .lock()
                .unwrap_or_else(|err| fatal!("Failed to lock level {}: {}", idx, err));

            let level = &mut *level;

            let world = &mut level.world;

            gfx::animation_system::update(&dt, world);
            controllable_system::update(&dt, actions, axes, world, self.input_cfg, cfg);

            let world = &mut level.world;

            // @Incomplete: level-specific gameplay update
            update_demo_entites(world, &dt);

            self.ground_collision_calc_system
                .update(world, &mut level.chunks);

            //movement_system::update(&dt, world);
            dumb_movement_system::update(&dt, world, rng);

            level.chunks.update(world);
        }
    }

    pub fn late_update(&mut self, evt_register: &mut Event_Register) {
        trace!("gameplay_system::late_update");

        self.foreach_active_level(|level| {
            level.world.notify_destroyed(evt_register);
            level.world.destroy_pending();
        });
    }

    pub fn realtime_update(
        &mut self,
        real_dt: &Duration,
        actions: &[Game_Action],
        axes: &Virtual_Axes,
        cfg: &cfg::Config,
    ) {
        trace!("gameplay_system::realtime_update");
        self.update_camera(real_dt, actions, axes, cfg);
    }

    fn update_camera(
        &mut self,
        real_dt: &Duration,
        actions: &[Game_Action],
        axes: &Virtual_Axes,
        cfg: &cfg::Config,
    ) {
        self.foreach_active_level(|level| {
            let movement = get_movement_from_input(axes, self.input_cfg, cfg);
            let v = {
                let camera_ctrl = level
                    .world
                    .get_component_mut::<C_Controllable>(level.cameras[level.active_camera]);
                if camera_ctrl.is_none() {
                    return;
                }

                let real_dt_secs = real_dt.as_secs_f32();
                let mut camera_ctrl = camera_ctrl.unwrap();
                let speed = camera_ctrl.speed.read(cfg);
                let velocity = movement * speed;
                let v = velocity * real_dt_secs;
                camera_ctrl.translation_this_frame = v;
                v
            };

            let camera = level
                .world
                .get_component_mut::<C_Camera2D>(level.cameras[level.active_camera])
                .unwrap();

            let sx = camera.transform.scale().x;
            let mut v = v * sx;

            let mut add_scale = Vec2f::new(0., 0.);
            const BASE_CAM_DELTA_ZOOM_PER_SCROLL: f32 = 0.2;

            if keyboard::is_key_pressed(keyboard::Key::LControl) {
                for action in actions {
                    match action {
                        (name, Action_Kind::Pressed)
                            if *name == String_Id::from("camera_zoom_up") =>
                        {
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

            if add_scale.magnitude2() > 0. {
                camera.transform.add_scale_v(add_scale);

                // Keep viewport centered
                let win_w = Cfg_Var::<i32>::new("engine/window/width", &cfg).read(&cfg);
                let win_h = Cfg_Var::<i32>::new("engine/window/height", &cfg).read(&cfg);
                v -= add_scale * 0.5 * v2!(win_w as f32, win_h as f32);
            }

            //return;
            camera.transform.translate_v(v);

            // DEBUG: center camera on player
            return;

            foreach_entity!(&level.world, +C_Controllable, ~C_Camera2D, |moved| {
                let pos = level
                    .world
                    .get_component::<C_Spatial2D>(moved)
                    .unwrap()
                    .transform
                    .position();

                let camera = level
                    .world
                    .get_component_mut::<C_Camera2D>(level.cameras[level.active_camera])
                    .unwrap();

                camera
                    .transform
                    .set_position_v(pos + Vec2f::new(-300., -300.));
            });
        });
    }

    #[cfg(debug_assertions)]
    pub fn step(&mut self, dt: &Duration, cfg: &cfg::Config, rng: &mut rand::Default_Rng) {
        // @Incomplete: probably should use previous frame actions
        self.update(dt, &[], &Virtual_Axes::default(), cfg, rng);
    }

    /*
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
        self.update(&dt, time, &actions, &axes, cfg);
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
            /*
            {
                let c = em.add_component::<collider::Collider>(entity);
                let width = (sw / n_frames) as f32;
                let height = sh as f32;
                c.shape = collider::Collision_Shape::Rect { width, height };
                //c.shape = collider::Collision_Shape::Circle {
                //radius: width.max(height) * 0.5,
                //};
                c.offset = -Vec2f::new(width * 0.5, height * 0.5);
            }
            */
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

    */
}

// @Temporary
fn update_demo_entites(ecs_world: &mut Ecs_World, dt: &Duration) {
    let dt_secs = dt.as_secs_f32();

    //let mut stream = new_entity_stream(ecs_world)
    //.require::<C_Controllable>()
    //.require::<C_Spatial2D>()
    //.build();
    //loop {
    //let entity = stream.next(ecs_world);
    //if entity.is_none() {
    //break;
    //}
    //let entity = entity.unwrap();
    //let ctrl = ecs_world.get_component::<C_Controllable>(entity).unwrap();
    //let transl = ctrl.translation_this_frame;
    //let spat = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();
    //spat.local_transform.translate_v(transl);
    //spat.velocity.x = transl.x;
    //spat.velocity.y = transl.y;
    //}

    foreach_entity_enumerate!(ecs_world, +C_Spatial2D, |entity, i| {
        let t = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();

        //if i == 1 {
        //t.velocity = Vec2f::new(-50.0, 0.);
        //}
        {
            //use ecs_engine::common::angle::deg;
            //let speed = 90.0;
            //if i == 1 {
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
    });
}

fn read_input_cfg(cfg: &cfg::Config) -> Input_Config {
    Input_Config {
        joy_deadzone: Cfg_Var::new("game/input/joystick/deadzone", cfg),
    }
}
