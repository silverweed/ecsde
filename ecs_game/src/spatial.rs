use inle_alloc::temp::*;
use inle_app::app::Engine_State;
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_query_new::{Ecs_Query, Update_Result};
use inle_ecs::ecs_world::{Component_Manager, Component_Updates, Ecs_World, Entity};
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
    query: Ecs_Query,
    pending_adds: Vec<Entity>,
    pending_removes: Vec<Entity>,
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
            query: Ecs_Query::default()
                .require::<C_Collider>()
                .require::<C_Spatial2D>(),
            pending_adds: vec![],
            pending_removes: vec![],
        }
    }

    pub fn init(&mut self, engine_state: &mut Engine_State) {}

    pub fn update_entity_components(
        &mut self,
        comp_mgr: &Component_Manager,
        entity: Entity,
        comp_updates: &Component_Updates,
    ) {
        trace!("world_chunks::update");

        match self
            .query
            .update(comp_mgr, entity, &comp_updates.added, &comp_updates.removed)
        {
            Update_Result::Added => {
                self.pending_adds.push(entity);
            }
            Update_Result::Removed => {
                self.pending_removes.push(entity);
            }
            _ => {}
        }
    }

    pub fn apply_pending_updates(&mut self, ecs_world: &Ecs_World, phys_world: &Physics_World) {
        trace!("world_chunks::apply_pending_updates");

        let pending_adds = std::mem::take(&mut self.pending_adds);
        let pending_removes = std::mem::take(&mut self.pending_removes);

        if let (Some(colliders), Some(spatials)) = (
            ecs_world.get_component_storage::<C_Collider>(),
            ecs_world.get_component_storage::<C_Spatial2D>(),
        ) {
            let colliders = colliders.lock_for_read();
            let spatials = spatials.lock_for_read();

            for entity in pending_removes {
                let collider = colliders.must_get(entity);
                for (cld, handle) in
                    phys_world.get_all_colliders_with_handles(collider.phys_body_handle)
                {
                    self.remove_collider(handle, cld.position, cld.shape.extent());
                }
            }

            for entity in pending_adds {
                let collider = colliders.must_get(entity);
                for (cld, handle) in
                    phys_world.get_all_colliders_with_handles(collider.phys_body_handle)
                {
                    // NOTE: here we cannot use cld.position because it's not been calculated yet.
                    let starting_pos = spatials.must_get(entity).transform.position() + cld.offset;
                    self.add_collider(handle, starting_pos, cld.shape.extent());
                }
            }
        }
    }

    pub fn n_chunks(&self) -> usize {
        self.chunks.len()
    }

    pub fn add_collider(&mut self, cld_handle: Collider_Handle, pos: Vec2f, extent: Vec2f) {
        let mut chunks = vec![];
        self.get_all_chunks_containing(pos, extent, &mut chunks);
        for coords in chunks {
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
        let mut chunks = vec![];
        self.get_all_chunks_containing(pos, extent, &mut chunks);
        for coords in chunks {
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

        let prev_coords = {
            let mut prev_coords = excl_temp_array(frame_alloc);
            self.get_all_chunks_containing(prev_pos, extent, &mut prev_coords);
            unsafe { prev_coords.into_read_only() }
        };
        let new_coords = {
            let mut new_coords = excl_temp_array(frame_alloc);
            self.get_all_chunks_containing(new_pos, extent, &mut new_coords);
            unsafe { new_coords.into_read_only() }
        };

        let mut all_chunks = excl_temp_array(frame_alloc);
        // Pre-allocate enough memory to hold all the chunks; then `chunks_to_add` starts at index 0,
        // while `chunks_to_remove` starts at index `new_coords.len()`.
        // This works because we can have at most `new_coords.len()` chunks to add and `prev_coords.len()`
        // chunks to remove.
        // @Robustness: this is likely UB in rust, we should do this in a safer way...
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
            .skip(chunks_to_remove_offset)
            .take(n_chunks_to_remove)
        {
            self.remove_collider_coords(cld_handle, *coord);
        }

        for coord in all_chunks
            .iter()
            .skip(chunks_to_add_offset)
            .take(n_chunks_to_add)
        {
            self.add_collider_coords(cld_handle, *coord);
        }
    }

    fn get_all_chunks_containing<T>(&self, pos: Vec2f, extent: Vec2f, coords: &mut T)
    where
        T: Extend<Chunk_Coords>,
    {
        trace!("get_all_chunks_containing");

        #[cfg(debug_assertions)]
        let mut chk_coords = vec![];

        // We need to @Cleanup the -extent*0.5 offset we need to apply and make it consistent throughout the game!
        let pos = pos - extent * 0.5;
        let coords_topleft = Chunk_Coords::from_pos(pos);
        coords.extend(Some(coords_topleft));

        #[cfg(debug_assertions)]
        chk_coords.push(coords_topleft);

        let coords_botright = Chunk_Coords::from_pos(pos + extent);

        // Note: we cycle y-major so the result is automatically sorted (as for Chunk_Coords::cmp)
        for y in 0..=coords_botright.y - coords_topleft.y {
            for x in 0..=coords_botright.x - coords_topleft.x {
                if x == 0 && y == 0 {
                    continue;
                }
                coords.extend(Some(Chunk_Coords::from_pos(
                    pos + v2!(x as f32 * CHUNK_WIDTH, y as f32 * CHUNK_HEIGHT),
                )));

                #[cfg(debug_assertions)]
                chk_coords.push(Chunk_Coords::from_pos(
                    pos + v2!(x as f32 * CHUNK_WIDTH, y as f32 * CHUNK_HEIGHT),
                ));
            }
        }

        #[cfg(debug_assertions)]
        {
            // Result should be sorted and deduped

            // @WaitForStable
            //debug_assert!(coords.iter().is_sorted());
            for i in 1..chk_coords.len() {
                debug_assert!(chk_coords[i] > chk_coords[i - 1]);
            }

            let mut deduped = chk_coords.clone();
            deduped.dedup();
            debug_assert!(chk_coords.len() == deduped.len());
        }
    }
}

impl Spatial_Accelerator<Collider_Handle> for World_Chunks {
    fn get_neighbours<R>(&self, pos: Vec2f, extent: Vec2f, result: &mut R)
    where
        R: Extend<Collider_Handle>,
    {
        let mut chunks = vec![];
        self.get_all_chunks_containing(pos, extent, &mut chunks);
        for coords in chunks {
            if let Some(chunk) = self.chunks.get(&coords) {
                result.extend(chunk.colliders.iter().copied());
            }
        }
    }
}

#[cfg(debug_assertions)]
impl World_Chunks {
    pub fn debug_draw(&self, painter: &mut Debug_Painter, _phys_world: &Physics_World) {
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

            // TODO: make this toggleable via a cfg var
            //for cld in &chunk.colliders {
            //let cld = phys_world.get_collider(*cld).unwrap();
            //painter.add_line(
            //inle_math::shapes::Line {
            //from: coords.to_world_pos(),
            //to: cld.position,
            //thickness: 1.,
            //},
            //colors::BLACK,
            //);
            //}
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
