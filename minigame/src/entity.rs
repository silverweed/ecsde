use crate::sprites::{self as anim_sprites, Anim_Sprite};
use inle_gfx::render::batcher::Batches;
use inle_gfx::render_window::Render_Window_Handle;
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use inle_physics::collider::{Collider, Collision_Shape, Phys_Data};
use inle_physics::phys_world::{Collider_Handle, Physics_Body_Handle, Physics_World};
use smallvec::SmallVec;

#[derive(Default, Clone)]
pub struct Entity {
    pub transform: Transform2D,
    pub velocity: Vec2f,
    pub sprites: SmallVec<[Anim_Sprite; 4]>,
    pub phys_body: Physics_Body_Handle,
}

impl Entity {
    pub fn new(sprite: Anim_Sprite) -> Self {
        Self {
            transform: Transform2D::default(),
            sprites: smallvec![sprite],
            phys_body: Physics_Body_Handle::default(),
            velocity: v2!(0., 0.),
        }
    }

    pub fn register_to_physics(&mut self, phys_world: &mut Physics_World) {
        // @Temporary
        let width = self.sprites[0].rect.width as f32;
        let height = self.sprites[0].rect.height as f32;
        let cld = Collider {
            shape: Collision_Shape::Rect { width, height },
            layer: 0,
            ..Default::default()
        };
        let phys_data = Phys_Data::default()
            .with_mass(1.)
            .with_restitution(0.9)
            .with_static_friction(0.5)
            .with_dyn_friction(0.3);
        let phys_body = phys_world.new_physics_body_with_rigidbody(cld, phys_data);
        self.phys_body = phys_body;
    }

    pub fn draw(&self, window: &mut Render_Window_Handle, batches: &mut Batches) {
        let mut sprites = self.sprites.clone();
        for sprite in &mut sprites {
            sprite.transform = self.transform.combine(&sprite.transform);
            anim_sprites::render_anim_sprite(window, batches, sprite);
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Entity_Handle {
    index: u32,
    gen: u32,
}

impl Default for Entity_Handle {
    fn default() -> Self {
        Self {
            index: u32::MAX,
            gen: INVALID_GEN,
        }
    }
}

const INVALID_GEN: u32 = u32::MAX;

#[derive(Default)]
pub struct Entity_Container {
    entities: Vec<Entity>,
    // i-th element of `entity_gens` maps to i-th element of `entities`.
    // Contains INVALID_GEN if the entity is invalid.
    entity_gens: Vec<u32>,
}

impl<'a> IntoIterator for &'a Entity_Container {
    type Item = &'a Entity;
    type IntoIter = std::slice::Iter<'a, Entity>;

    fn into_iter(self) -> Self::IntoIter {
        self.entities.iter()
    }
}

impl<'a> IntoIterator for &'a mut Entity_Container {
    type Item = &'a mut Entity;
    type IntoIter = std::slice::IterMut<'a, Entity>;

    fn into_iter(self) -> Self::IntoIter {
        self.entities.iter_mut()
    }
}

impl Entity_Container {
    pub fn push(&mut self, entity: Entity) -> Entity_Handle {
        debug_assert_eq!(self.entities.len(), self.entity_gens.len());
        assert!(self.entities.len() < u32::MAX as usize - 2);

        if let Some((idx, old_gen)) = self.first_free_slot() {
            self.entities[idx as usize] = entity;
            self.entity_gens[idx as usize] = old_gen + 1;
            Entity_Handle {
                index: idx,
                gen: old_gen + 1,
            }
        } else {
            self.entities.push(entity);
            self.entity_gens.push(0);
            Entity_Handle {
                index: self.entities.len() as u32 - 1,
                gen: 0,
            }
        }
    }

    pub fn remove(&mut self, handle: Entity_Handle) {
        debug_assert_eq!(self.entities.len(), self.entity_gens.len());
        assert!(handle.index < self.entities.len() as _);

        if self.entity_gens[handle.index as usize] == handle.gen {
            self.entity_gens[handle.index as usize] = INVALID_GEN;
            // NOTE: currently we don't actually delete the entity.
            debug_assert!(!std::mem::needs_drop::<Entity>());
        } else {
            lwarn!(
                "Failed to remove entity {:?}: handle is obsolete or invalid.",
                handle
            );
        }
    }

    pub fn clear(&mut self) {
        self.entities.clear();
        self.entity_gens.clear();
    }

    pub fn get(&self, handle: Entity_Handle) -> Option<&Entity> {
        debug_assert_eq!(self.entities.len(), self.entity_gens.len());

        if handle.index as usize >= self.entities.len() {
            return None;
        }

        if self.entity_gens[handle.index as usize] == handle.gen {
            Some(&self.entities[handle.index as usize])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, handle: Entity_Handle) -> Option<&mut Entity> {
        debug_assert_eq!(self.entities.len(), self.entity_gens.len());

        if handle.index as usize >= self.entities.len() {
            return None;
        }

        if self.entity_gens[handle.index as usize] == handle.gen {
            Some(&mut self.entities[handle.index as usize])
        } else {
            None
        }
    }

    pub fn no_entity_was_ever_added(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn n_live(&self) -> usize {
        self.entity_gens
            .iter()
            .filter(|&&g| g != INVALID_GEN)
            .count()
    }

    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter {
        self.into_iter()
    }

    // Returns (index, gen) of first free slot, or None if there are none.
    fn first_free_slot(&self) -> Option<(u32, u32)> {
        for (i, &gen) in self.entity_gens.iter().enumerate() {
            if gen == INVALID_GEN {
                return Some((i as _, gen));
            }
        }
        None
    }
}

// @Temporary: the dumbest spatial accelerator possible
impl inle_physics::spatial::Spatial_Accelerator<Collider_Handle> for Entity_Container {
    fn get_neighbours<R>(
        &self,
        pos: Vec2f,
        extent: Vec2f,
        phys_world: &Physics_World,
        result: &mut R,
    ) where
        R: Extend<Collider_Handle>,
    {
        result.extend(self.iter().flat_map(|e| {
            phys_world
                .get_all_phys_body_colliders_with_handles(e.phys_body)
                .map(|(_, hdl)| hdl)
        }));
    }
}
