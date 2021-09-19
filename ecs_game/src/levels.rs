use crate::spatial::World_Chunks;
use inle_common::stringid::String_Id;
use inle_ecs::ecs_query::Ecs_Query;
use inle_ecs::ecs_world::{Ecs_World, Entity};
use inle_gfx::components::C_Camera2D;
use inle_gfx::light::Lights;
use inle_math::vector::Vec2f;
use inle_physics::phys_world::Physics_World;
use std::sync::{Arc, Mutex, MutexGuard};

// A Level is what gets loaded and unloaded
pub struct Level {
    pub id: String_Id,
    pub world: Ecs_World,
    pub chunks: World_Chunks,
    pub cameras: Vec<Entity>,
    pub active_camera: usize, // index inside 'cameras'
    pub lights: Lights,
    pub phys_world: Physics_World,
}

impl Level {
    // @Temporary: we need to better decide how to handle cameras
    pub fn get_camera_transform(&self) -> inle_math::transform::Transform2D {
        let query = Ecs_Query::new(&self.world).read::<C_Camera2D>();
        let cam_entity = query.entities()[0];
        let cams = query.storages().begin_read::<C_Camera2D>();
        cams.must_get(cam_entity).transform.clone()
    }

    // @Temporary
    pub fn move_camera_to(&mut self, pos: Vec2f) {
        let query = Ecs_Query::new(&self.world).read::<C_Camera2D>();
        let cam_entity = query.entities()[0];
        let mut cams = query.storages().begin_write::<C_Camera2D>();
        cams.must_get_mut(cam_entity).transform.set_position_v(pos);
    }
}

#[derive(Default)]
pub struct Levels {
    pub loaded_levels: Vec<Arc<Mutex<Level>>>,
    pub active_levels: Vec<usize>, // indices inside 'loaded_levels'
}

impl Levels {
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
}
