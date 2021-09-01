use inle_alloc::temp::*;
use inle_app::app::Engine_State;
use inle_ecs::ecs_world::{Ecs_World, Entity, Evt_Entity_Destroyed};
use inle_events::evt_register::{with_cb_data, wrap_cb_data, Event_Callback_Data};
use inle_math::vector::Vec2f;
use inle_physics::collider::C_Collider;
use inle_physics::phys_world::{Collider_Handle, Physics_World};
use inle_physics::spatial::Spatial_Accelerator;
use std::cmp::Ordering;
use std::collections::HashMap;

#[cfg(debug_assertions)]
use {inle_debug::painter::Debug_Painter, std::collections::HashSet};

// @Speed: tune these numbers
const CHUNK_WIDTH: f32 = 200.;
const CHUNK_HEIGHT: f32 = 200.;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chunk_Coords {
    pub x: i32,
    pub y: i32,
}

impl PartialOrd for Chunk_Coords {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Chunk_Coords {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.y.cmp(&other.y) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.x.cmp(&other.x),
        }
    }
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
    pub colliders: Vec<Collider_Handle>,
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

    pub fn update(&mut self, ecs_world: &Ecs_World, phys_world: &Physics_World) {
        let mut to_remove = vec![];
        with_cb_data(&mut self.to_destroy, |to_destroy: &mut Vec<Entity>| {
            for &entity in to_destroy.iter() {
                if let Some(collider) = ecs_world.get_component::<C_Collider>(entity) {
                    for (cld, handle) in phys_world.get_all_colliders_with_handles(collider.handle)
                    {
                        to_remove.push((handle, cld.position, cld.shape.extent()));
                    }
                }
            }
            to_destroy.clear();
        });
        for (cld, pos, extent) in to_remove {
            self.remove_collider(cld, pos, extent);
        }
    }

    pub fn n_chunks(&self) -> usize {
        self.chunks.len()
    }

    pub fn add_collider(&mut self, cld_handle: Collider_Handle, pos: Vec2f, extent: Vec2f) {
        for coords in self.get_all_chunks_containing(pos, extent) {
            self.add_collider_coords(cld_handle, coords);
        }
    }

    fn add_collider_coords(&mut self, cld_handle: Collider_Handle, coords: Chunk_Coords) {
        let chunk = self
            .chunks
            .entry(coords)
            .or_insert_with(World_Chunk::default);
        debug_assert!(
            !chunk.colliders.contains(&cld_handle),
            "Duplicate collider {:?} in chunk {:?}!",
            cld_handle,
            coords
        );
        chunk.colliders.push(cld_handle);
    }

    pub fn remove_collider(&mut self, cld_handle: Collider_Handle, pos: Vec2f, extent: Vec2f) {
        for coords in self.get_all_chunks_containing(pos, extent) {
            self.remove_collider_coords(cld_handle, coords);
        }
    }

    fn remove_collider_coords(&mut self, cld_handle: Collider_Handle, coords: Chunk_Coords) {
        let chunk = self.chunks.get_mut(&coords).unwrap_or_else(|| {
            fatal!(
                "Collider {:?} should be in chunk {:?}, but that chunk does not exist.",
                cld_handle,
                coords
            )
        });
        let idx = chunk.colliders.iter().position(|&c| c == cld_handle);
        if let Some(idx) = idx {
            chunk.colliders.remove(idx);
            if chunk.colliders.is_empty() {
                self.chunks.remove(&coords);
            }
        } else {
            lerr!(
                "Collider {:?} not found in expected chunk {:?}.",
                cld_handle,
                coords
            );
        }
    }

    pub fn update_collider(
        &mut self,
        cld_handle: Collider_Handle,
        prev_pos: Vec2f,
        new_pos: Vec2f,
        extent: Vec2f,
        frame_alloc: &mut Temp_Allocator,
    ) {
        trace!("world_chunks::update_collider");

        let prev_coords = self.get_all_chunks_containing(prev_pos, extent);
        let new_coords = self.get_all_chunks_containing(new_pos, extent);

        let mut all_chunks = excl_temp_array(frame_alloc);
        // Pre-allocate enough memory to hold all the chunks; then `chunks_to_add` starts at index 0,
        // while `chunks_to_remove` starts at index `new_coords.len()`.
        // This works because we can have at most `new_coords.len()` chunks to add and `prev_coords.len()`
        // chunks to remove.
        unsafe {
            all_chunks.alloc_additional_uninit(new_coords.len() + prev_coords.len());
        }

        let mut n_chunks_to_add = 0;
        let mut n_chunks_to_remove = 0;
        let chunks_to_add_offset = 0;
        let chunks_to_remove_offset = new_coords.len();

        // Find chunks to add and to remove in O(n).
        // This algorithm assumes that both prev_coords and new_coords are sorted and deduped.
        let mut p_idx = 0;
        let mut n_idx = 0;
        while p_idx < prev_coords.len() && n_idx < new_coords.len() {
            let pc = prev_coords[p_idx];
            let nc = new_coords[n_idx];
            match pc.cmp(&nc) {
                Ordering::Less => {
                    all_chunks[chunks_to_remove_offset + n_chunks_to_remove] = pc;
                    n_chunks_to_remove += 1;
                    p_idx += 1;
                }
                Ordering::Greater => {
                    all_chunks[chunks_to_add_offset + n_chunks_to_add] = nc;
                    n_chunks_to_add += 1;
                    n_idx += 1;
                }
                Ordering::Equal => {
                    p_idx += 1;
                    n_idx += 1;
                }
            }
        }
        if p_idx < prev_coords.len() {
            let diff = prev_coords.len() - p_idx;
            for i in 0..diff {
                all_chunks[chunks_to_remove_offset + n_chunks_to_remove + i] =
                    prev_coords[p_idx + i];
            }
            n_chunks_to_remove += diff;
        } else if n_idx < new_coords.len() {
            let diff = new_coords.len() - n_idx;
            for i in 0..diff {
                all_chunks[chunks_to_add_offset + n_chunks_to_add + i] = new_coords[n_idx + i];
            }
            n_chunks_to_add += diff;
        }

        #[cfg(debug_assertions)]
        {
            let to_remove = all_chunks
                .iter()
                .cloned()
                .skip(chunks_to_remove_offset)
                .take(n_chunks_to_remove)
                .collect::<HashSet<_>>();
            let to_add = all_chunks
                .iter()
                .cloned()
                .skip(chunks_to_add_offset)
                .take(n_chunks_to_add)
                .collect::<HashSet<_>>();
            debug_assert_eq!(to_remove.intersection(&to_add).count(), 0);
        }

        for coord in all_chunks
            .iter()
            .skip(chunks_to_add_offset)
            .take(n_chunks_to_add)
        {
            self.add_collider_coords(cld_handle, *coord);
        }

        for coord in all_chunks
            .iter()
            .skip(chunks_to_remove_offset)
            .take(n_chunks_to_remove)
        {
            self.remove_collider_coords(cld_handle, *coord);
        }
    }

    fn get_all_chunks_containing(&self, pos: Vec2f, extent: Vec2f) -> Vec<Chunk_Coords> {
        let mut coords = vec![];

        // We need to @Cleanup the -extent*0.5 offset we need to apply and make it consistent throughout the game!
        let pos = pos - extent * 0.5;
        let coords_topleft = Chunk_Coords::from_pos(pos);
        coords.push(coords_topleft);

        let coords_botright = Chunk_Coords::from_pos(pos + extent);

        // Note: we cycle y-major so the result is automatically sorted (as for Chunk_Coords::cmp)
        for y in 0..=coords_botright.y - coords_topleft.y {
            for x in 0..=coords_botright.x - coords_topleft.x {
                if x == 0 && y == 0 {
                    continue;
                }
                coords.push(Chunk_Coords::from_pos(
                    pos + v2!(x as f32 * CHUNK_WIDTH, y as f32 * CHUNK_HEIGHT),
                ));
            }
        }

        #[cfg(debug_assertions)]
        {
            // @WaitForStable
            //debug_assert!(coords.iter().is_sorted());
            for i in 1..coords.len() {
                debug_assert!(coords[i] > coords[i - 1]);
            }

            let mut deduped = coords.clone();
            deduped.dedup();
            debug_assert_eq!(coords.len(), deduped.len());
        }

        coords
    }
}

impl Spatial_Accelerator<Collider_Handle> for World_Chunks {
    fn get_neighbours<R>(&self, pos: Vec2f, extent: Vec2f, result: &mut R)
    where
        R: Extend<Collider_Handle>,
    {
        for coords in self.get_all_chunks_containing(pos, extent) {
            if let Some(chunk) = self.chunks.get(&coords) {
                result.extend(chunk.colliders.iter().copied());
            }
        }
    }
}

#[cfg(debug_assertions)]
impl World_Chunks {
    pub fn debug_draw(&self, painter: &mut Debug_Painter) {
        use inle_common::colors;
        use inle_common::paint_props::Paint_Properties;
        use inle_math::transform::Transform2D;

        if self.chunks.is_empty() {
            return;
        }

        let max_colliders = self
            .chunks
            .iter()
            .map(|(_, chk)| chk.colliders.len())
            .max()
            .unwrap_or(0) as f32;

        for (coords, chunk) in &self.chunks {
            let world_pos = v2!(coords.to_world_pos().x, coords.to_world_pos().y);
            let col = colors::lerp_col(
                colors::rgba(0, 150, 0, 100),
                colors::rgba(150, 0, 0, 100),
                chunk.colliders.len() as f32 / max_colliders,
            );
            painter.add_rect(
                v2!(CHUNK_WIDTH, CHUNK_HEIGHT),
                &Transform2D::from_pos(world_pos),
                Paint_Properties {
                    color: col,
                    border_color: colors::darken(col, 0.7),
                    border_thick: (CHUNK_WIDTH / 50.).max(5.),
                    ..Default::default()
                },
            );
            painter.add_text(
                &format!("{},{}: {}", coords.x, coords.y, chunk.colliders.len()),
                world_pos + v2!(10., 5.),
                (CHUNK_WIDTH as u16 / 10).max(20),
                colors::rgba(50, 220, 0, 250),
            );
        }
    }
}

#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    fn chunk_coords_ord() {
        assert!(Chunk_Coords { x: 0, y: 0 } < Chunk_Coords { x: 1, y: 0 });
        assert!(Chunk_Coords { x: 1, y: 0 } < Chunk_Coords { x: 0, y: 1 });
        assert!(Chunk_Coords { x: 1, y: 1 } < Chunk_Coords { x: 2, y: 1 });
        assert!(Chunk_Coords { x: 2, y: 1 } < Chunk_Coords { x: 1, y: 2 });
    }
}
