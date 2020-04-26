use ecs_engine::common::vector::Vec2f;
use ecs_engine::core::app::Engine_State;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::ecs_world::{Ecs_World, Entity, Evt_Entity_Destroyed};
use ecs_engine::events::evt_register::{with_cb_data, wrap_cb_data, Event_Callback_Data};
use std::collections::HashMap;

#[cfg(debug_assertions)]
use ecs_engine::debug::painter::Debug_Painter;

const CHUNK_WIDTH: f32 = 1000.;
const CHUNK_HEIGHT: f32 = 1000.;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chunk_Coords {
    pub x: i32,
    pub y: i32,
}

impl Chunk_Coords {
    pub fn from_pos(pos: Vec2f) -> Self {
        Self {
            x: (pos.x / CHUNK_WIDTH).floor() as i32,
            y: (pos.y / CHUNK_HEIGHT).floor() as i32,
        }
    }

    pub fn to_world_pos(self) -> Vec2f {
        Vec2f {
            x: self.x as f32 * CHUNK_WIDTH,
            y: self.y as f32 * CHUNK_HEIGHT,
        }
    }
}

pub struct World_Chunks {
    chunks: HashMap<Chunk_Coords, World_Chunk>,
    to_destroy: Event_Callback_Data,
}

#[derive(Default, Debug)]
pub struct World_Chunk {
    pub entities: Vec<Entity>,
}

impl World_Chunks {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            to_destroy: wrap_cb_data(Vec::<Entity>::new()),
        }
    }

    pub fn init(&mut self, engine_state: &mut Engine_State) {
        engine_state
            .systems
            .evt_register
            .subscribe::<Evt_Entity_Destroyed>(
                Box::new(|entity, to_destroy| {
                    with_cb_data(to_destroy.unwrap(), |to_destroy: &mut Vec<Entity>| {
                        to_destroy.push(entity);
                    });
                }),
                Some(self.to_destroy.clone()),
            );
    }

    pub fn update(&mut self, ecs_world: &Ecs_World) {
        let mut to_remove = vec![];
        with_cb_data(&mut self.to_destroy, |to_destroy: &mut Vec<Entity>| {
            for &entity in to_destroy.iter() {
                if let Some(spatial) = ecs_world.get_component::<C_Spatial2D>(entity) {
                    to_remove.push((entity, spatial.transform.position()));
                }
            }
            to_destroy.clear();
        });
        for (entity, pos) in to_remove {
            self.remove_entity(entity, pos);
        }
    }

    pub fn add_entity(&mut self, entity: Entity, pos: Vec2f) {
        let coords = Chunk_Coords::from_pos(pos);
        self.add_entity_coords(entity, coords);
    }

    fn add_entity_coords(&mut self, entity: Entity, coords: Chunk_Coords) {
        let chunk = self
            .chunks
            .entry(coords)
            .or_insert_with(World_Chunk::default);
        debug_assert!(
            !chunk.entities.contains(&entity),
            "Duplicate entity {:?} in chunk {:?}!",
            entity,
            coords
        );
        chunk.entities.push(entity);
    }

    pub fn remove_entity(&mut self, entity: Entity, pos: Vec2f) {
        let coords = Chunk_Coords::from_pos(pos);
        self.remove_entity_coords(entity, coords);
    }

    fn remove_entity_coords(&mut self, entity: Entity, coords: Chunk_Coords) {
        let chunk = self.chunks.get_mut(&coords).unwrap_or_else(|| {
            fatal!(
                "Entity {:?} should be in chunk {:?}, but that chunk does not exist.",
                entity,
                coords
            )
        });
        let idx = chunk.entities.iter().position(|&e| e == entity);
        if let Some(idx) = idx {
            chunk.entities.remove(idx);
        } else {
            lerr!(
                "Entity {:?} not found in expected chunk {:?}.",
                entity,
                coords
            );
        }
    }

    pub fn update_entity(&mut self, entity: Entity, prev_pos: Vec2f, new_pos: Vec2f) {
        let prev_coords = Chunk_Coords::from_pos(prev_pos);
        let new_coords = Chunk_Coords::from_pos(new_pos);

        if prev_coords == new_coords {
            return;
        }

        self.remove_entity_coords(entity, prev_coords);
        self.add_entity_coords(entity, new_coords);
    }
}

#[cfg(debug_assertions)]
impl World_Chunks {
    pub fn debug_draw(&self, painter: &mut Debug_Painter) {
        use ecs_engine::common::colors;
        use ecs_engine::common::transform::Transform2D;
        use ecs_engine::gfx::paint_props::Paint_Properties;

        if self.chunks.is_empty() {
            return;
        }

        let max_entities = self
            .chunks
            .iter()
            .map(|(_, chk)| chk.entities.len())
            .max()
            .unwrap_or(0) as f32;

        for (coords, chunk) in &self.chunks {
            let world_pos = v2!(coords.to_world_pos().x, coords.to_world_pos().y);
            let col = colors::lerp_col(
                colors::rgba(0, 150, 0, 100),
                colors::rgba(150, 0, 0, 100),
                chunk.entities.len() as f32 / max_entities,
            );
            painter.add_rect(
                v2!(CHUNK_WIDTH, CHUNK_HEIGHT),
                &Transform2D::from_pos(world_pos),
                Paint_Properties {
                    color: col,
                    border_color: colors::darken(col, 0.7),
                    border_thick: 20.,
                    ..Default::default()
                },
            );
            painter.add_text(
                &format!("{},{}: {}", coords.x, coords.y, chunk.entities.len()),
                world_pos + v2!(10., 5.),
                100,
                colors::rgba(50, 220, 0, 250),
            );
        }
    }
}
