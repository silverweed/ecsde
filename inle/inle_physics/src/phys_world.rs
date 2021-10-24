use super::collider::Collider;
use inle_alloc::gen_alloc::{Generational_Allocator, Generational_Index};
use inle_math::vector::Vec2f;
use smallvec::SmallVec;
use std::collections::HashMap;

pub type Collider_Handle = Generational_Index;
pub type Physics_Body_Handle = Generational_Index;

const INITIAL_SIZE: usize = 64;

#[derive(Copy, Clone, Debug, Default)]
pub struct Phys_Data {
    pub inv_mass: f32,
    pub restitution: f32,
    pub static_friction: f32,
    pub dyn_friction: f32,
}

/// A Physics_Body can contain any number of Colliders, and it's what is associated
/// to a single Entity. Note that the association between a Physics_Body and its
/// Colliders is done externally, and the Physics_World is unaware of it.
#[derive(Default, Debug, Clone)]
pub struct Physics_Body {
    pub rigidbody_colliders: SmallVec<[(Collider_Handle, Phys_Data); 1]>,
    pub trigger_colliders: Vec<Collider_Handle>,
}

impl Physics_Body {
    pub fn all_colliders(&self) -> impl Iterator<Item = Collider_Handle> + '_ {
        Physics_Body_Cld_Iter { body: self, i: 0 }
    }
}

struct Physics_Body_Cld_Iter<'a> {
    body: &'a Physics_Body,
    i: usize,
}

impl Iterator for Physics_Body_Cld_Iter<'_> {
    type Item = Collider_Handle;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.i as usize;
        self.i += 1;
        if i < self.body.rigidbody_colliders.len() {
            Some(self.body.rigidbody_colliders[i].0)
        } else {
            self.body
                .trigger_colliders
                .get(i - self.body.rigidbody_colliders.len())
                .copied()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Collision_Data {
    pub other_collider: Collider_Handle,
    pub info: Collision_Info,
}

#[derive(Debug, Clone)]
pub struct Collision_Info {
    pub penetration: f32,
    pub normal: Vec2f,
}

pub struct Physics_World {
    cld_alloc: Generational_Allocator,
    /// Indexed by a Collider_Handle's index. Contains the index into `colliders`.
    /// This double indirection allows us to always keep `colliders` compact and
    /// keep iteration over it fast, sacrificing lookup and removal speed.
    cld_index_table: Vec<usize>,
    pub(super) colliders: Vec<Collider>,

    bodies_alloc: Generational_Allocator,
    /// Indexed by a Physics_Body_Handle's index.
    pub(super) bodies: Vec<Physics_Body>,

    /// Contains all collisions for this frame.
    pub(super) collisions: HashMap<Collider_Handle, SmallVec<[Collision_Data; 4]>>,
}

impl Physics_World {
    pub fn new() -> Self {
        Self {
            cld_alloc: Generational_Allocator::new(INITIAL_SIZE),
            cld_index_table: vec![],
            colliders: vec![],
            bodies_alloc: Generational_Allocator::new(INITIAL_SIZE),
            bodies: vec![],
            collisions: HashMap::default(),
        }
    }

    #[inline]
    pub fn is_valid_collider_handle(&self, handle: Collider_Handle) -> bool {
        self.cld_alloc.is_valid(handle)
    }

    pub fn new_physics_body(&mut self) -> Physics_Body_Handle {
        let handle = self.bodies_alloc.allocate();
        if self.bodies.len() <= handle.index as usize {
            self.bodies
                .resize(handle.index as usize + 1, Physics_Body::default());
        } else {
            self.bodies
                .insert(handle.index as usize, Physics_Body::default());
        }
        handle
    }

    pub fn new_physics_body_with_rigidbody(
        &mut self,
        cld: Collider,
        phys_data: Phys_Data,
    ) -> Physics_Body_Handle {
        let cld_handle = self.add_collider(cld);
        let handle = self.new_physics_body();
        let body = self.get_physics_body_mut(handle).unwrap();
        body.rigidbody_colliders.push((cld_handle, phys_data));
        handle
    }

    #[inline]
    pub fn get_physics_body(&self, handle: Physics_Body_Handle) -> Option<&Physics_Body> {
        if !self.bodies_alloc.is_valid(handle) {
            return None;
        }

        assert!(
            (handle.index as usize) < self.bodies.len(),
            "Handle {:?} is out of bounds for bodies array of len {}!",
            handle,
            self.bodies.len()
        );

        Some(&self.bodies[handle.index as usize])
    }

    #[inline]
    pub fn get_physics_body_mut(
        &mut self,
        handle: Physics_Body_Handle,
    ) -> Option<&mut Physics_Body> {
        if !self.bodies_alloc.is_valid(handle) {
            return None;
        }

        assert!(
            (handle.index as usize) < self.bodies.len(),
            "Handle {:?} is out of bounds for bodies array of len {}!",
            handle,
            self.bodies.len()
        );

        Some(&mut self.bodies[handle.index as usize])
    }

    #[inline]
    pub fn add_collider(&mut self, mut cld: Collider) -> Collider_Handle {
        let handle = self.cld_alloc.allocate();
        debug_assert!((handle.index as usize) < std::u32::MAX as usize);
        let index = self.colliders.len();
        cld.handle = handle;
        self.colliders.push(cld);
        self.cld_index_table.push(index);
        handle
    }

    /// Note: this is a O(n) operation, so use sparingly.
    pub fn remove_collider(&mut self, handle: Collider_Handle) {
        if !self.cld_alloc.is_valid(handle) {
            lwarn!("Tried to remove invalid collider {:?}", handle);
            return;
        }

        assert!(
            (handle.index as usize) < self.cld_index_table.len(),
            "Handle {:?} is out of bounds for cld_index_table of len {}!",
            handle,
            self.cld_index_table.len()
        );

        let index = self.cld_index_table[handle.index as usize];
        let swapped_index = self.colliders.len() - 1;
        self.colliders.swap_remove(index);

        // Patch swapped index into index table
        *self
            .cld_index_table
            .iter_mut()
            .find(|idx| **idx == swapped_index)
            .unwrap() = index;

        self.cld_alloc.deallocate(handle);
    }

    #[inline]
    pub fn get_collider(&self, handle: Collider_Handle) -> Option<&Collider> {
        if !self.cld_alloc.is_valid(handle) {
            return None;
        }

        assert!(
            (handle.index as usize) < self.cld_index_table.len(),
            "Handle {:?} is out of bounds for cld_index_table of len {}!",
            handle,
            self.cld_index_table.len()
        );
        let index = self.cld_index_table[handle.index as usize];

        debug_assert!(index < self.colliders.len());
        Some(&self.colliders[index])
    }

    #[inline]
    pub fn get_collider_mut(&mut self, handle: Collider_Handle) -> Option<&mut Collider> {
        if !self.cld_alloc.is_valid(handle) {
            return None;
        }

        assert!(
            (handle.index as usize) < self.cld_index_table.len(),
            "Handle {:?} is out of bounds for cld_index_table of len {}!",
            handle,
            self.cld_index_table.len()
        );
        let index = self.cld_index_table[handle.index as usize];

        debug_assert!(index < self.colliders.len());
        Some(&mut self.colliders[index])
    }

    #[inline]
    pub fn get_all_colliders(
        &self,
        handle: Physics_Body_Handle,
    ) -> impl Iterator<Item = &Collider> {
        let body_handle = self.get_physics_body(handle);
        let mut iter = None;
        if let Some(body) = body_handle {
            iter = Some(
                body.all_colliders()
                    .map(move |h| self.get_collider(h).unwrap()),
            );
        }

        std::iter::from_fn(move || {
            if let Some(iter) = iter.as_mut() {
                iter.next()
            } else {
                None
            }
        })
    }

    #[inline]
    pub fn get_all_colliders_with_handles(
        &self,
        handle: Physics_Body_Handle,
    ) -> impl Iterator<Item = (&Collider, Collider_Handle)> {
        let body_handle = self.get_physics_body(handle);
        let mut iter = None;
        if let Some(body) = body_handle {
            iter = Some(
                body.all_colliders()
                    .map(move |h| (self.get_collider(h).unwrap(), h)),
            );
        }

        std::iter::from_fn(move || {
            if let Some(iter) = iter.as_mut() {
                iter.next()
            } else {
                None
            }
        })
    }

    #[inline]
    pub fn get_first_rigidbody_collider(&self, handle: Physics_Body_Handle) -> Option<&Collider> {
        self.get_physics_body(handle).and_then(|body| {
            body.rigidbody_colliders
                .get(0)
                .and_then(|(h, _)| self.get_collider(*h))
        })
    }

    #[inline]
    pub fn get_rigidbody_colliders(
        &self,
        handle: Physics_Body_Handle,
    ) -> impl Iterator<Item = &Collider> + '_ {
        let mut maybe_iter = self.get_physics_body(handle).map(move |body| {
            body.rigidbody_colliders
                .iter()
                .map(move |(h, _)| self.get_collider(*h))
        });
        std::iter::from_fn(move || {
            if let Some(iter) = &mut maybe_iter {
                iter.next()?
            } else {
                None
            }
        })
    }

    pub(super) fn clear_collisions(&mut self) {
        self.collisions.clear();
    }

    pub(super) fn add_collision(
        &mut self,
        cld_a: Collider_Handle,
        cld_b: Collider_Handle,
        info: &Collision_Info,
    ) {
        self.collisions
            .entry(cld_a)
            .or_insert_with(SmallVec::default)
            .push(Collision_Data {
                other_collider: cld_b,
                info: Collision_Info {
                    normal: -info.normal,
                    ..*info
                },
            });
        self.collisions
            .entry(cld_b)
            .or_insert_with(SmallVec::default)
            .push(Collision_Data {
                other_collider: cld_a,
                info: info.clone(),
            });
    }

    #[inline]
    pub fn get_collisions(&self, cld: Collider_Handle) -> &[Collision_Data] {
        static EMPTY_COLLISIONS: [Collision_Data; 0] = [];
        if let Some(cls) = self.collisions.get(&cld) {
            &cls
        } else {
            &EMPTY_COLLISIONS
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::collider::Collision_Shape;
    use super::*;

    #[test]
    fn add_collider() {
        let mut phys_world = Physics_World::new();
        let c = Collider {
            shape: Collision_Shape::Circle { radius: 24. },
            ..Default::default()
        };
        let h1 = phys_world.add_collider(c);
        let c = Collider {
            shape: Collision_Shape::Rect {
                width: 2.,
                height: 3.,
            },
            ..Default::default()
        };
        let h2 = phys_world.add_collider(c);

        assert_eq!(
            phys_world.get_collider(h1).unwrap().shape,
            Collision_Shape::Circle { radius: 24. }
        );
        assert_eq!(
            phys_world.get_collider(h2).unwrap().shape,
            Collision_Shape::Rect {
                width: 2.,
                height: 3.
            }
        );
    }

    #[test]
    fn remove_collider() {
        let mut phys_world = Physics_World::new();

        assert!(phys_world
            .get_collider(Collider_Handle { index: 0, gen: 0 })
            .is_none());

        let c = Collider {
            shape: Collision_Shape::Circle { radius: 24. },
            ..Default::default()
        };
        let h1 = phys_world.add_collider(c);
        let c = Collider {
            shape: Collision_Shape::Rect {
                width: 2.,
                height: 3.,
            },
            ..Default::default()
        };
        let h2 = phys_world.add_collider(c.clone());

        phys_world.remove_collider(h1);

        assert!(phys_world.get_collider(h1).is_none());
        assert!(phys_world.get_collider(h2).is_some());

        let h3 = phys_world.add_collider(c);
        assert!(phys_world.get_collider(h1).is_none());
        assert!(phys_world.get_collider(h3).is_some());
    }
}
