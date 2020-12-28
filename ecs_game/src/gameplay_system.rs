#![allow(warnings)] // @Temporary

use super::levels::{Level, Levels};
use super::systems::camera_system;
use super::systems::controllable_system::{self, C_Controllable};
use super::systems::dumb_movement_system;
use super::systems::entity_preview_system::{self, C_Entity_Preview};
use super::systems::gravity_system;
use super::systems::ground_collision_calculation_system::Ground_Collision_Calculation_System;
use super::systems::pixel_collision_system::Pixel_Collision_System;
use crate::gfx;
use crate::input_utils::{get_movement_from_input, Input_Config};
use crate::load::load_system;
use crate::movement_system;
use crate::spatial::World_Chunks;
use crate::Game_Resources;
use inle_app::app::Engine_State;
use inle_cfg::{self, Cfg_Var};
use inle_common::colors;
use inle_common::stringid::String_Id;
use inle_core::env::Env_Info;
use inle_core::{rand, time};
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::{Ecs_World, Entity};
use inle_ecs::entity_stream::new_entity_stream;
use inle_events::evt_register::Event_Register;
use inle_gfx::components::{C_Animated_Sprite, C_Camera2D, C_Renderable};
use inle_gfx::particles::Particle_Manager;
use inle_gfx::render::batcher::Batches;
use inle_gfx::render_window::Render_Window_Handle;
use inle_input::axes::Virtual_Axes;
use inle_input::input_state::{Action_Kind, Game_Action, Input_State};
use inle_input::keyboard;
use inle_math::rect::Rect;
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use inle_physics::collider;
use inle_resources::gfx::{tex_path, Gfx_Resources};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

#[derive(Default, Copy, Clone)]
pub struct Gameplay_System_Config {
    pub n_entities_to_spawn: usize,
}

pub struct Gameplay_System {
    pub levels: Levels,

    pub input_cfg: Input_Config,
    cfg: Gameplay_System_Config,

    ground_collision_calc_system: Ground_Collision_Calculation_System,
    pub pixel_collision_system: Pixel_Collision_System,

    pub cursor_entity: Option<Entity>,

    #[cfg(debug_assertions)]
    debug_data: Debug_Data,

    // @Cleanup: this is pretty ugly here
    camera_on_player: Cfg_Var<bool>,
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
            levels: Levels::default(),
            input_cfg: Input_Config::default(),
            cfg: Gameplay_System_Config::default(),
            ground_collision_calc_system: Ground_Collision_Calculation_System::new(),
            pixel_collision_system: Pixel_Collision_System::default(),
            cursor_entity: None,
            #[cfg(debug_assertions)]
            debug_data: Debug_Data::default(),
            camera_on_player: Cfg_Var::default(),
        }
    }

    pub fn init(
        &mut self,
        gres: &mut Gfx_Resources,
        engine_state: &mut Engine_State,
        gs_cfg: Gameplay_System_Config,
    ) -> inle_common::Maybe_Error {
        self.input_cfg = read_input_cfg(&engine_state.config);
        self.cfg = gs_cfg;
        self.ground_collision_calc_system.init(engine_state);
        self.camera_on_player = Cfg_Var::new("game/camera/on_player", &engine_state.config);

        Ok(())
    }

    // @Temporary
    pub fn load_test_level(
        &mut self,
        engine_state: &mut Engine_State,
        game_res: &mut Game_Resources,
        level_batches: &mut HashMap<String_Id, Batches>,
    ) {
        let level_id = sid!("test");
        let mut level = load_system::level_load_sync(level_id, engine_state, game_res, self.cfg);

        level.chunks.init(engine_state);

        // @Temporary
        //self.cursor_entity = Some(create_cursor_entity(
        //    &mut game_res.gfx,
        //    &mut engine_state.env,
        //    &mut level.world,
        //));

        self.levels.loaded_levels.push(Arc::new(Mutex::new(level)));
        self.levels
            .active_levels
            .push(self.levels.loaded_levels.len() - 1);

        level_batches.insert(level_id, Batches::default());
        engine_state.systems.particle_mgrs.insert(
            level_id,
            Particle_Manager::new(
                &mut engine_state.shader_cache,
                &engine_state.env,
                &engine_state.config,
            ),
        );

        #[cfg(debug_assertions)]
        {
            engine_state.debug_systems.new_debug_painter_for_level(
                level_id,
                &mut game_res.gfx,
                &engine_state.env,
            );
        }

        // @Temporary
        /*
        {
            use inle_gfx::particles;
            use inle_math::angle;

            let texture = game_res
                .gfx
                .load_texture(&tex_path(&engine_state.env, "yv.png"));
            for i in 0..100 {
                let props = particles::Particle_Props {
                    n_particles: 100,
                    spread: angle::deg(50.0),
                    initial_speed: 50.0..200.0,
                    initial_rotation: angle::deg(0.0)..angle::deg(180.0),
                    lifetime: Duration::from_millis(100)..Duration::from_secs(3),
                    acceleration: -50.0,
                    texture,
                    color: colors::BLUE,
                    ..Default::default()
                };
                let rng = &mut engine_state.rng;
                let particle_mgr = engine_state
                    .systems
                    .particle_mgrs
                    .get_mut(&level_id)
                    .unwrap();
                let handle = particle_mgr.add_particles(&props, rng);
                particle_mgr
                    .get_particles_mut(handle)
                    .transform
                    .set_position(i as f32 * 20.0, 0.0);
                particle_mgr
                    .get_particles_mut(handle)
                    .transform
                    .set_rotation(inle_math::angle::rad(i as f32 * 0.2));
            }
        }*/
    }

    // @Temporary
    pub fn unload_test_level(
        &mut self,
        engine_state: &mut Engine_State,
        level_batches: &mut HashMap<String_Id, Batches>,
    ) {
        let level_id = sid!("test");

        if let Some(idx) = self
            .levels
            .loaded_levels
            .iter()
            .position(|lv| lv.lock().unwrap().id == level_id)
        {
            self.levels.loaded_levels.remove(idx);
            if let Some(idxx) = self.levels.active_levels.iter().position(|i| *i == idx) {
                self.levels.active_levels.remove(idxx);
            }
        }

        #[cfg(debug_assertions)]
        {
            engine_state.debug_systems.painters.remove(&level_id);
        }

        level_batches.remove(&level_id);
        engine_state.systems.particle_mgrs.remove(&level_id);
    }

    pub fn update(
        &mut self,
        dt: &Duration,
        engine_state: &mut Engine_State,
        rsrc: &mut Game_Resources,
        window: &Render_Window_Handle,
    ) {
        trace!("gameplay_system::update");

        let axes = &engine_state.input_state.processed.virtual_axes;
        let actions = &engine_state.input_state.processed.game_actions;

        #[cfg(debug_assertions)]
        {
            self.debug_data.latest_frame_actions = actions.to_vec();
        }

        let rng = &mut engine_state.rng;
        let cfg = &engine_state.config;

        ///// Update all game systems in all worlds /////
        // Note: inlining foreach_active_levels because we don't want to borrow self.
        let levels = &self.levels;
        let input_cfg = self.input_cfg;
        let ground_collision_calc_system = &mut self.ground_collision_calc_system;
        let frame_alloc = &mut engine_state.frame_alloc;
        let gres = &mut rsrc.gfx;
        let env = &engine_state.env;
        let shader_cache = &mut engine_state.shader_cache;

        levels.foreach_active_level(|level| {
            let world = &mut level.world;

            inle_app::animation_system::update(&dt, world);
            controllable_system::update(&dt, actions, axes, world, input_cfg, cfg);

            let world = &mut level.world;

            // @Incomplete: level-specific gameplay update
            update_demo_entites(world, &dt);

            ground_collision_calc_system.update(world, &mut level.phys_world, &mut level.chunks);

            gravity_system::update(&dt, world, cfg);
            //movement_system::update(&dt, world);
            dumb_movement_system::update(&dt, world, &mut level.phys_world, rng);

            gfx::multi_sprite_animation_system::update(&dt, world, frame_alloc);
            let camera = level.get_camera().transform;
            entity_preview_system::update(
                &mut level.world,
                &mut level.phys_world,
                window,
                gres,
                shader_cache,
                env,
                &camera,
                &actions,
                cfg,
            );

            level.chunks.update(&mut level.world, &level.phys_world);
        });
    }

    pub fn late_update(&mut self, evt_register: &mut Event_Register) {
        trace!("gameplay_system::late_update");

        self.levels.foreach_active_level(|level| {
            level.world.notify_destroyed(evt_register);
            level.world.destroy_pending();
        });
    }

    pub fn realtime_update(
        &mut self,
        real_dt: &Duration,
        engine_state: &Engine_State,
        window: &Render_Window_Handle,
    ) {
        trace!("gameplay_system::realtime_update");

        if self.camera_on_player.read(&engine_state.config) {
            self.update_camera(real_dt, engine_state, window);
        } else {
            self.update_free_camera(real_dt, &engine_state.input_state, &engine_state.config);
        }
    }

    fn update_camera(
        &mut self,
        dt: &Duration,
        engine_state: &Engine_State,
        window: &Render_Window_Handle,
    ) {
        self.levels.foreach_active_level(|level| {
            let world = &mut level.world;

            camera_system::update(dt, world, window, &engine_state.config);
        });
    }

    fn update_free_camera(
        &mut self,
        dt: &Duration,
        input_state: &Input_State,
        cfg: &inle_cfg::Config,
    ) {
        self.levels.foreach_active_level(|level| {
            let movement =
                get_movement_from_input(&input_state.processed.virtual_axes, self.input_cfg, cfg);
            let v = {
                let camera_ctrl = level
                    .world
                    .get_component_mut::<C_Controllable>(level.cameras[level.active_camera]);
                if camera_ctrl.is_none() {
                    return;
                }

                let dt_secs = dt.as_secs_f32();
                let mut camera_ctrl = camera_ctrl.unwrap();
                let speed = camera_ctrl.speed.read(cfg);
                let velocity = movement * speed;
                let v = velocity * dt_secs;
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

            for action in &input_state.processed.game_actions {
                match action {
                    (name, Action_Kind::Pressed) if *name == sid!("camera_zoom_up") => {
                        add_scale.x -= BASE_CAM_DELTA_ZOOM_PER_SCROLL * sx;
                        add_scale.y = add_scale.x;
                    }
                    (name, Action_Kind::Pressed) if *name == sid!("camera_zoom_down") => {
                        add_scale.x += BASE_CAM_DELTA_ZOOM_PER_SCROLL * sx;
                        add_scale.y = add_scale.x;
                    }
                    _ => (),
                }
            }

            if add_scale.magnitude2() > 0. {
                camera.transform.add_scale_v(add_scale);

                // Keep viewport centered
                let win_w = Cfg_Var::<i32>::new("engine/window/width", cfg).read(cfg);
                let win_h = Cfg_Var::<i32>::new("engine/window/height", cfg).read(cfg);
                v -= add_scale * 0.5 * v2!(win_w as f32, win_h as f32);
            }

            //return;
            camera.transform.translate_v(v);
        });
    }

    #[cfg(debug_assertions)]
    pub fn step(
        &mut self,
        dt: &Duration,
        engine_state: &mut Engine_State,
        resources: &mut Game_Resources,
        window: &Render_Window_Handle,
    ) {
        // @Incomplete: probably should use previous frame actions
        self.update(dt, engine_state, resources, window);
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
                let (sw, sh) = inle_gfx::render::get_texture_size(rsrc.get_texture(rend.texture));
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

    foreach_entity_enumerate!(ecs_world, +C_Spatial2D, +C_Controllable, |entity, i| {
        let t = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();

        //if i == 1 {
        //t.velocity = Vec2f::new(-50.0, 0.);
        //}
        {
            use inle_math::angle::deg;
            let speed = 90.0;
            //if i == 1 {
            //t.transform.rotate(deg(dt_secs * speed));
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

fn read_input_cfg(cfg: &inle_cfg::Config) -> Input_Config {
    Input_Config {
        joy_deadzone: Cfg_Var::new("game/input/joystick/deadzone", cfg),
    }
}

fn create_cursor_entity(gres: &mut Gfx_Resources, env: &Env_Info, world: &mut Ecs_World) -> Entity {
    let entity = world.new_entity();
    world.add_component(
        entity,
        C_Renderable::new_with_diffuse(gres, env, "drill.png")
            .with_normals(gres, env, "drill_n.png")
            .with_z_index(10)
            .with_color(colors::rgba(255, 255, 255, 150)),
    );
    let spat = world.add_component(entity, C_Spatial2D::default());
    spat.transform.set_scale(0.2, 0.2); // @Cleanup
    world.add_component(entity, C_Entity_Preview::default());
    entity
}
