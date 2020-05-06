use ecs_engine::common::stringid::String_Id;
use ecs_engine::gfx::light::Lights;
use ecs_engine::ecs::ecs_world::{Entity, Ecs_World};
use crate::spatial::World_Chunks;
use ecs_engine::ecs::components::gfx::C_Camera2D;
use ecs_engine::common::vector::Vec2f;
use std::sync::{Arc, Mutex, MutexGuard};

// A Level is what gets loaded and unloaded
pub struct Level {
    pub id: String_Id,
    pub world: Ecs_World,
    pub chunks: World_Chunks,
    pub cameras: Vec<Entity>,
    pub active_camera: usize, // index inside 'cameras'
    pub lights: Lights,
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

