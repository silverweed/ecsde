use crate::ecs_world::Entity;
use anymap::any::UncheckedAnyExt;
use anymap::Map;
use std::any::type_name;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct Component_Manager {
    storages: Map<dyn Component_Storage_Interface>,
}

impl Component_Manager {
    pub fn new() -> Self {
        Self {
            storages: Map::new(),
        }
    }

    #[inline]
    pub fn has_component<T: 'static>(&self, entity: Entity) -> bool {
        if let Some(storage) = self.get_component_storage::<T>() {
            storage.has_component(entity)
        } else {
            false
        }
    }

    #[inline]
    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<Component_Read<T>> {
        if let Some(storage) = self.get_component_storage::<T>() {
            storage.get_component(entity)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_component_mut<T: 'static>(&self, entity: Entity) -> Option<Component_Write<T>> {
        if let Some(storage) = self.get_component_storage::<T>() {
            storage.get_component_mut(entity)
        } else {
            None
        }
    }

    #[inline]
    pub fn add_component<T: 'static>(&mut self, entity: Entity, data: T) {
        let storage = self
            .storages
            .entry::<Component_Storage<T>>()
            .or_insert_with(Component_Storage::<T>::default);

        let mut components = storage.components.write().unwrap();
        let cur_components_len = components.len();

        // Ensure the entity doesn't have this component already
        {
            let slot = storage.entity_comp_index.get(entity.index as usize);
            if slot.copied().flatten().is_some() {
                fatal!(
                    "Component {:?} added twice to entity {:?}!",
                    type_name::<T>(),
                    entity
                );
            }

            if storage.entity_comp_index.len() <= entity.index as usize {
                storage
                    .entity_comp_index
                    .resize(entity.index as usize + 1, None);
            }
            storage
                .entity_comp_index
                .insert(entity.index as usize, Some(cur_components_len));
        }

        #[cfg(debug_assertions)]
        {
            if storage.entity_comp_generation.len() <= entity.index as usize {
                storage
                    .entity_comp_generation
                    .resize(entity.index as usize + 1, 0);
            }
            storage
                .entity_comp_generation
                .insert(entity.index as usize, entity.gen);
        }

        components.push(data);
    }

    #[inline]
    pub fn remove_component<T: 'static>(&mut self, entity: Entity) {
        if let Some(storage) = self.get_component_storage_mut::<T>() {
            storage.remove_component(entity);
        } else {
            lerr!(
                "Tried to remove inexisting component {:?} from entity {:?}",
                type_name::<T>(),
                entity
            );
        }
    }

    #[inline]
    pub fn remove_all_components(&mut self, entity: Entity) {
        for raw_storage in self.storages.as_mut().iter_mut() {
            raw_storage.remove_component(entity);
        }
    }

    #[inline]
    pub fn get_component_storage<T: 'static>(&self) -> Option<&Component_Storage<T>> {
        self.storages.get::<Component_Storage<T>>()
    }

    #[inline]
    pub fn get_component_storage_mut<T: 'static>(&mut self) -> Option<&mut Component_Storage<T>> {
        self.storages.get_mut::<Component_Storage<T>>()
    }

    #[inline]
    pub fn read_component_storage<T: 'static>(&self) -> Option<Component_Storage_Read<T>> {
        self.storages
            .get::<Component_Storage<T>>()
            .map(|storage| storage.lock_for_read())
    }

    #[inline]
    pub fn write_component_storage<T: 'static>(&self) -> Option<Component_Storage_Write<T>> {
        self.storages
            .get::<Component_Storage<T>>()
            .map(|storage| storage.lock_for_write())
    }
}

#[cfg(debug_assertions)]
impl Component_Manager {
    pub fn get_comp_name_list_for_entity(&self, entity: Entity) -> Vec<&'static str> {
        self.storages
            .as_ref()
            .iter()
            .filter_map(|storage| {
                if storage.has_component(entity) {
                    Some(storage.comp_name())
                } else {
                    None
                }
            })
            .collect()
    }
}

pub struct Component_Storage<T> {
    /// Indexed by entity_comp_index
    components: RwLock<Vec<T>>,

    /// Indexed by entity index.
    entity_comp_index: Vec<Option<usize>>,

    #[cfg(debug_assertions)]
    /// Keeps track of the generation of the entity when the component was last added to ensure consistency.
    /// Indexed by entity index
    entity_comp_generation: Vec<inle_alloc::gen_alloc::Gen_Type>,
}

impl<T> Component_Storage<T> {
    pub fn get_component(&self, entity: Entity) -> Option<Component_Read<T>> {
        if let Some(slot) = self.entity_comp_index.get(entity.index as usize) {
            slot.map(|idx| {
                let comps = self.components.read().unwrap();
                Component_Read { lock: comps, idx }
            })
        } else {
            None
        }
    }

    pub fn get_component_mut(&self, entity: Entity) -> Option<Component_Write<T>> {
        if let Some(slot) = self.entity_comp_index.get(entity.index as usize) {
            slot.map(move |idx| {
                let comps = self.components.write().unwrap();
                Component_Write { lock: comps, idx }
            })
        } else {
            None
        }
    }
}

impl<T> Default for Component_Storage<T> {
    fn default() -> Self {
        Self {
            components: RwLock::new(vec![]),
            entity_comp_index: vec![],
            #[cfg(debug_assertions)]
            entity_comp_generation: vec![],
        }
    }
}

pub struct Component_Read<'l, T> {
    lock: RwLockReadGuard<'l, Vec<T>>,
    idx: usize,
}

impl<'l, T> std::ops::Deref for Component_Read<'l, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.lock[self.idx]
    }
}

pub struct Component_Write<'l, T> {
    lock: RwLockWriteGuard<'l, Vec<T>>,
    idx: usize,
}

impl<'l, T> std::ops::Deref for Component_Write<'l, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.lock[self.idx]
    }
}

impl<'l, T> std::ops::DerefMut for Component_Write<'l, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lock[self.idx]
    }
}

pub struct Component_Storage_Read<'a, T> {
    components: RwLockReadGuard<'a, Vec<T>>,
    entity_comp_index: &'a [Option<usize>],
    #[cfg(debug_assertions)]
    entity_comp_generation: &'a [inle_alloc::gen_alloc::Gen_Type],
}

pub struct Component_Storage_Write<'a, T> {
    components: RwLockWriteGuard<'a, Vec<T>>,
    entity_comp_index: &'a [Option<usize>],
    #[cfg(debug_assertions)]
    entity_comp_generation: &'a [inle_alloc::gen_alloc::Gen_Type],
}

impl<T> Component_Storage<T> {
    pub fn lock_for_read(&self) -> Component_Storage_Read<'_, T> {
        trace!("Component_Storage::lock_for_read");

        Component_Storage_Read {
            components: self.components.read().unwrap(),
            entity_comp_index: &self.entity_comp_index,
            #[cfg(debug_assertions)]
            entity_comp_generation: &self.entity_comp_generation,
        }
    }

    pub fn lock_for_write(&self) -> Component_Storage_Write<'_, T> {
        trace!("Component_Storage::lock_for_write");

        Component_Storage_Write {
            components: self.components.write().unwrap(),
            entity_comp_index: &self.entity_comp_index,
            #[cfg(debug_assertions)]
            entity_comp_generation: &self.entity_comp_generation,
        }
    }
}

impl<T> Component_Storage_Read<'_, T> {
    #[inline]
    pub fn get(&self, entity: Entity) -> Option<&T> {
        trace!("Component_Storage_Read::get");

        #[cfg(debug_assertions)]
        {
            assert_gen_consistency(
                self.entity_comp_index,
                self.entity_comp_generation,
                entity,
                type_name::<T>(),
            );
        }
        self.entity_comp_index[entity.index as usize].map(|idx| &self.components[idx])
    }

    #[inline]
    pub fn must_get(&self, entity: Entity) -> &T {
        trace!("Component_Storage_Read::must_get");

        #[cfg(debug_assertions)]
        {
            assert_gen_consistency(
                self.entity_comp_index,
                self.entity_comp_generation,
                entity,
                type_name::<T>(),
            );
        }
        let idx = self.entity_comp_index[entity.index as usize].unwrap();
        &self.components[idx]
    }
}

impl<T> Component_Storage_Write<'_, T> {
    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        trace!("Component_Storage_Write::get_mut");

        #[cfg(debug_assertions)]
        {
            assert_gen_consistency(
                self.entity_comp_index,
                self.entity_comp_generation,
                entity,
                type_name::<T>(),
            );
        }
        self.entity_comp_index[entity.index as usize].map(move |idx| &mut self.components[idx])
    }

    #[inline]
    pub fn must_get_mut(&mut self, entity: Entity) -> &mut T {
        trace!("Component_Storage_Write::must_get_mut");

        #[cfg(debug_assertions)]
        {
            assert_gen_consistency(
                self.entity_comp_index,
                self.entity_comp_generation,
                entity,
                type_name::<T>(),
            );
        }
        let idx = self.entity_comp_index[entity.index as usize].unwrap();
        &mut self.components[idx]
    }
}

impl<T: 'static> Component_Storage<T> {
    #[inline]
    pub fn has_component(&self, entity: Entity) -> bool {
        if let Some(slot) = self.entity_comp_index.get(entity.index as usize) {
            slot.is_some()
        } else {
            false
        }
    }
}

impl<'a, T: 'a> IntoIterator for &'a Component_Storage_Read<'a, T> {
    type IntoIter = Component_Storage_Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            inner: self.components.iter(),
        }
    }
}

pub struct Component_Storage_Iter<'a, T> {
    inner: std::slice::Iter<'a, T>,
}

impl<'a, T: 'a> Iterator for Component_Storage_Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, T: 'a> IntoIterator for &'a mut Component_Storage_Write<'a, T> {
    type IntoIter = Component_Storage_Iter_Mut<'a, T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            inner: self.components.iter_mut(),
        }
    }
}

pub struct Component_Storage_Iter_Mut<'a, T> {
    inner: std::slice::IterMut<'a, T>,
}

impl<'a, T: 'a> Iterator for Component_Storage_Iter_Mut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(debug_assertions)]
fn assert_gen_consistency(
    comp_index: &[Option<usize>],
    comp_gen: &[inle_alloc::gen_alloc::Gen_Type],
    entity: Entity,
    type_name: &'static str,
) {
    debug_assert_eq!(comp_index.len(), comp_gen.len());
    debug_assert_eq!(
                entity.gen,
                comp_gen[entity.index as usize],
                "Entity {:?} is not the same as the one contained in Component_Storage<{:?}> ({:?})! Probably it was destroyed and recreated but its components were not updated properly!", entity, type_name, comp_gen[entity.index as usize]);
}

// This trait is required to define common methods for all storages
// so we can iterate on them in an untyped fashion and still be
// able to do stuff on them (e.g. remove_all_components)
pub(super) trait Component_Storage_Interface: anymap::any::Any {
    fn remove_component(&mut self, entity: Entity);

    #[cfg(debug_assertions)]
    fn has_component(&self, entity: Entity) -> bool;

    #[cfg(debug_assertions)]
    fn comp_name(&self) -> &'static str;
}

impl<T: 'static> Component_Storage_Interface for Component_Storage<T> {
    fn remove_component(&mut self, entity: Entity) {
        if let Some(slot) = self.entity_comp_index.get_mut(entity.index as usize) {
            *slot = None;
        } else {
            lerr!(
                "Tried to remove inexisting component {:?} from entity {:?}",
                type_name::<T>(),
                entity
            );
        }
    }

    // @Cleanup: I wonder if this is the best way to do this.
    // Should we just expose the trait and have all methods be
    // trait methods? But would that mean that we'd have to go through
    // a vtable all the time? I guess not if we have the concrete type?
    // Investigate on this.
    #[cfg(debug_assertions)]
    fn has_component(&self, entity: Entity) -> bool {
        Component_Storage::<T>::has_component(self, entity)
    }

    #[cfg(debug_assertions)]
    fn comp_name(&self) -> &'static str {
        // Note: we assume that type_name returns a full path like foo::bar::Type, although that's not guaranteed.
        // This won't break if that's not the case anyway.
        let full_name = type_name::<T>();
        let base_name = full_name.rsplit(':').next().unwrap_or(full_name);
        base_name
    }
}

//
// The following traits are to make anymap::Map work with our trait
//

impl UncheckedAnyExt for dyn Component_Storage_Interface {
    #[inline]
    unsafe fn downcast_ref_unchecked<T: 'static>(&self) -> &T {
        &*(self as *const Self as *const T)
    }

    #[inline]
    unsafe fn downcast_mut_unchecked<T: 'static>(&mut self) -> &mut T {
        &mut *(self as *mut Self as *mut T)
    }

    #[inline]
    unsafe fn downcast_unchecked<T: 'static>(self: Box<Self>) -> Box<T> {
        Box::from_raw(Box::into_raw(self) as *mut T)
    }
}

impl<T: 'static> anymap::any::IntoBox<dyn Component_Storage_Interface> for Component_Storage<T> {
    #[inline]
    fn into_box(self) -> Box<dyn Component_Storage_Interface> {
        Box::new(self)
    }
}
