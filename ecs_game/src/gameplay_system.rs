#![allow(warnings)] // @Temporary

use super::levels::{Level, Levels};
use super::systems::animation_system;
use super::systems::camera_system;
use super::systems::controllable_system::{self, C_Controllable};
use super::systems::free_camera_system;
use super::systems::movement_system;
use crate::systems::ai;
use crate::systems::ground_detection_system;
use inle_alloc::temp;
//use super::systems::dumb_movement_system;
use super::systems::gravity_system;
//use super::systems::ground_collision_calculation_system::Ground_Collision_Calculation_System;
//use super::systems::pixel_collision_system::Pixel_Collision_System;
use crate::gfx;
use crate::input_utils::Input_Config;
use crate::load::load_system;
use crate::spatial::World_Chunks;
use crate::systems::interface::{Game_System, Realtime_Update_Args, Update_Args};
use crate::Game_Resources;
use inle_app::app::Engine_State;
use inle_cfg::{self, Cfg_Var};
use inle_common::colors;
use inle_common::stringid::String_Id;
use inle_core::env::Env_Info;
use inle_core::{rand, time};
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::{Component_Manager, Component_Updates, Ecs_World, Entity};
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

    systems: Vec<Box<dyn Game_System>>,

    // This is a special system so we keep it separate (as it's a pain in the ass to downcast stuff
    // in Rust)
    movement_system: movement_system::Movement_System,

    #[cfg(debug_assertions)]
    debug_data: Debug_Data,

    // @Temporary: we should create a C_Particle_Emitter to treat emitters like entities
    test_particle_emitter: inle_gfx::particles::Particle_Emitter_Handle,
}

#[cfg(debug_assertions)]
#[derive(Default)]
struct Debug_Data {
    pub latest_frame_actions: Vec<Game_Action>,
    pub latest_frame_axes: Virtual_Axes,
}

fn create_systems(engine_state: &Engine_State) -> Vec<Box<dyn Game_System>> {
    vec![
        Box::new(gravity_system::Gravity_System::new()),
        Box::new(ground_detection_system::Ground_Detection_System::new()),
        Box::new(ai::test_ai_system::Test_Ai_System::new()),
        Box::new(controllable_system::Controllable_System::new(
            &engine_state.config,
        )),
        Box::new(camera_system::Camera_System::new(&engine_state.config)),
        Box::new(free_camera_system::Free_Camera_System::new(
            &engine_state.config,
        )),
        Box::new(animation_system::Animation_System::new()),
    ]
}

impl Gameplay_System {
    pub fn new(engine_state: &Engine_State) -> Gameplay_System {
        Gameplay_System {
            levels: Levels::default(),
            input_cfg: Input_Config::default(),
            cfg: Gameplay_System_Config::default(),
            systems: create_systems(engine_state),
            movement_system: movement_system::Movement_System::new(),
            #[cfg(debug_assertions)]
            debug_data: Debug_Data::default(),
            test_particle_emitter: inle_gfx::particles::Particle_Emitter_Handle::default(),
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

        Ok(())
    }

    // @Temporary
    pub fn load_test_level(
        &mut self,
        window: &mut Render_Window_Handle,
        engine_state: &mut Engine_State,
        game_res: &mut Game_Resources,
        level_batches: &mut HashMap<String_Id, Batches>,
        cvars: &crate::game_state::CVars,
    ) {
        let level_id = sid!("test");
        let mut level =
            load_system::level_load_sync(level_id, engine_state, game_res, self.cfg, cvars);

        self.levels.loaded_levels.push(Arc::new(Mutex::new(level)));
        self.levels
            .active_levels
            .push(self.levels.loaded_levels.len() - 1);

        level_batches.insert(level_id, Batches::default());
        engine_state.systems.particle_mgrs.insert(
            level_id,
            Particle_Manager::new(
                &mut game_res.shader_cache,
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
        {
            use inle_gfx::particles;
            use inle_math::angle;

            let texture = game_res
                .gfx
                .load_texture(&tex_path(&engine_state.env, "white_circle.png"));
            let props = particles::Particle_Props {
                n_particles: 12,
                spread: angle::deg(70.0),
                initial_speed: 15.0..50.0,
                initial_scale: 2.0..12.0,
                initial_rotation: angle::deg(0.0)..angle::deg(180.0),
                lifetime: Duration::from_millis(100)..Duration::from_millis(800),
                acceleration: -50.0,
                texture,
                color: colors::DARK_ORANGE,
                ..Default::default()
            };
            let rng = &mut engine_state.rng;
            let particle_mgr = engine_state
                .systems
                .particle_mgrs
                .get_mut(&level_id)
                .unwrap();
            let handle = particle_mgr.add_emitter(
                window,
                &props,
                Rect::from_center_size(v2!(0., 0.), v2!(10., 20.)),
                rng,
            );
            particle_mgr
                .get_emitter_mut(handle)
                .transform
                .set_position(-250., 180.0);
            particle_mgr
                .get_emitter_mut(handle)
                .transform
                .set_rotation(angle::deg(-90.0));

            self.test_particle_emitter = handle;

            /*
            let texture = game_res
                .gfx
                .load_texture(&tex_path(&engine_state.env, "yv.png"));
            for i in 0..20 {
                let props = particles::Particle_Props {
                    n_particles: 100,
                    spread: angle::deg(50.0),
                    initial_speed: 50.0..200.0,
                    initial_scale: 1.0..20.0,
                    initial_rotation: angle::deg(0.0)..angle::deg(180.0),
                    lifetime: Duration::from_millis(100)..Duration::from_secs(3),
                    acceleration: -50.0,
                    texture,
                    color: colors::PINK,
                    ..Default::default()
                };
                let rng = &mut engine_state.rng;
                let particle_mgr = engine_state
                    .systems
                    .particle_mgrs
                    .get_mut(&level_id)
                    .unwrap();
                let handle = particle_mgr.add_emitter(window, &props, rng);
                particle_mgr
                    .get_emitter_mut(handle)
                    .transform
                    .set_position(i as f32 * 20.0, 0.0);
                particle_mgr
                    .get_emitter_mut(handle)
                    .transform
                    .set_rotation(inle_math::angle::rad(i as f32 * 0.2));
            }
            */
        }
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
        let levels = &self.levels;
        let input_cfg = self.input_cfg;
        let gres = &mut rsrc.gfx;
        let shader_cache = &mut rsrc.shader_cache;
        let test_emitter_handle = self.test_particle_emitter;
        let systems = &mut self.systems;

        levels.foreach_active_level(|level| {
            let world = &mut level.world;

            let mut update_args = Update_Args {
                dt: *dt,
                ecs_world: world,
                phys_world: &mut level.phys_world,
                engine_state,
                input_cfg: &input_cfg,
            };
            for system in systems.iter() {
                system.update(&mut update_args);
            }

            //level.chunks.update(&mut level.world, &level.phys_world);

            // @Temporary DEBUG (this only works if we only have 1 test level)
            //let particle_mgr = particle_mgrs.get_mut(&level.id).unwrap();
            //particle_mgr
            //.get_emitter_mut(test_emitter_handle)
            //.transform
            //.rotate(inle_math::angle::deg(dt.as_secs_f32() * 30.0));
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
        window: &Render_Window_Handle,
        engine_state: &mut Engine_State,
    ) {
        trace!("gameplay_system::realtime_update");

        let mut systems = &mut self.systems;
        let input_cfg = &self.input_cfg;

        self.levels.foreach_active_level(|level| {
            let mut update_args = Realtime_Update_Args {
                dt: *real_dt,
                window,
                engine_state,
                ecs_world: &mut level.world,
                cameras: &mut level.cameras,
                active_camera: level.active_camera,
                input_cfg,
            };
            for system in systems.iter_mut() {
                system.realtime_update(&mut update_args);
            }
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
    }*/

    pub fn update_pending_component_updates<F>(&mut self, mut on_comp_update_cb: F)
    where
        F: FnMut(&HashMap<Entity, Component_Updates>, &Component_Manager),
    {
        let mut systems = &mut self.systems;
        let movement_system = &mut self.movement_system;
        self.levels.foreach_active_level(|level| {
            let pending_updates = level
                .world
                .get_and_flush_pending_component_updates_for_systems();

            on_comp_update_cb(&pending_updates, &level.world.component_manager);

            for (entity, comp_updates) in pending_updates {
                for system in systems.iter_mut() {
                    for query in system.get_queries_mut() {
                        query.update(
                            &level.world.component_manager,
                            entity,
                            &comp_updates.added,
                            &comp_updates.removed,
                        );
                    }
                }

                // Movement system
                for query in movement_system.get_queries_mut() {
                    query.update(
                        &level.world.component_manager,
                        entity,
                        &comp_updates.added,
                        &comp_updates.removed,
                    );
                }

                // Chunks
                level.chunks.update_entity_components(
                    &level.world.component_manager,
                    entity,
                    &comp_updates,
                );
            }
        });
    }

    pub fn pre_update_physics(&mut self) {
        trace!("gameplay_system::pre_update_physics");

        self.levels.foreach_active_level(|level| {
            level
                .chunks
                .apply_pending_updates(&level.world, &level.phys_world);
        });
    }

    pub fn update_physics(&mut self, update_dt: Duration, frame_alloc: &mut temp::Temp_Allocator) {
        trace!("gameplay_system::update_physics");

        let movement_system = &mut self.movement_system;

        self.levels.foreach_active_level(|level| {
            let mut moved = temp::excl_temp_array(frame_alloc);
            movement_system.update_physics(
                &update_dt,
                &mut level.world,
                &level.phys_world,
                &mut moved,
            );

            let moved = unsafe { moved.into_read_only() };
            for mov in &moved {
                level.chunks.update_collider(
                    mov.handle,
                    mov.prev_pos,
                    mov.new_pos,
                    mov.extent,
                    frame_alloc,
                );
            }
        });
    }
}

fn read_input_cfg(cfg: &inle_cfg::Config) -> Input_Config {
    Input_Config {
        joy_deadzone: Cfg_Var::new("game/input/joystick/deadzone", cfg),
    }
}
