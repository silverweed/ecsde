use crate::ecs_world::Entity;
use anymap::any::UncheckedAnyExt;
use anymap::Map;
use std::any::type_name;

pub struct Component_Manager {
    storages: Map<dyn Component_Storage_Interface>,
}

#[cfg(debug_assertions)]
fn assert_gen_consistency<T>(storage: &Component_Storage<T>, entity: Entity) {
    debug_assert_eq!(
        storage.entity_comp_index.len(),
        storage.entity_comp_generation.len()
    );
    debug_assert_eq!(
                entity.gen,
                storage.entity_comp_generation[entity.index as usize],
                "Entity {:?} is not the same as the one contained in Component_Storage<{:?}> ({:?})! Probably it was destroyed and recreated but its components were not updated properly!", entity, type_name::<T>(), storage.entity_comp_generation[entity.index as usize]);
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
    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
        if let Some(storage) = self.get_component_storage::<T>() {
            storage.get_component(entity)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        if let Some(storage) = self.get_component_storage_mut::<T>() {
            storage.get_component_mut(entity)
        } else {
            None
        }
    }

    #[inline]
    pub fn add_component<T: 'static + Default>(&mut self, entity: Entity, data: T) -> &mut T {
        let storage = self
            .storages
            .entry::<Component_Storage<T>>()
            .or_insert_with(Component_Storage::<T>::default);

        let cur_components_len = storage.components.len();

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

            storage
                .entity_comp_index
                .insert(entity.index as usize, Some(cur_components_len + 1));
        }

        #[cfg(debug_assertions)]
        {
            storage
                .entity_comp_generation
                .insert(entity.index as usize, entity.gen);
        }

        storage.components.push(data);
        &mut storage.components[cur_components_len]
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
    pub fn get_components<T: 'static>(&self) -> impl Iterator<Item = &T> {
        if let Some(storage) = self.get_component_storage::<T>() {
            storage.components.iter()
        } else {
            [].iter()
        }
    }

    #[inline]
    pub fn get_components_mut<T: 'static>(&mut self) -> impl Iterator<Item = &mut T> {
        if let Some(storage) = self.get_component_storage_mut::<T>() {
            storage.components.iter_mut()
        } else {
            [].iter_mut()
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

#[derive(Default)]
pub struct Component_Storage<T> {
    /// Indexed by entity_comp_index
    components: Vec<T>,

    /// Indexed by entity index.
    entity_comp_index: Vec<Option<usize>>,

    #[cfg(debug_assertions)]
    /// Keeps track of the generation of the entity when the component was last added to ensure consistency.
    /// Indexed by entity index
    entity_comp_generation: Vec<inle_alloc::gen_alloc::Gen_Type>,
}

impl<T: 'static> Component_Storage<T> {
    #[inline]
    pub fn has_component(&self, entity: Entity) -> bool {
        if let Some(slot) = self.entity_comp_index.get(entity.index as usize) {
            #[cfg(debug_assertions)]
            {
                assert_gen_consistency(self, entity);
            }

            slot.is_some()
        } else {
            false
        }
    }

    #[inline]
    pub fn get_component(&self, entity: Entity) -> Option<&T> {
        if let Some(slot) = self.entity_comp_index.get(entity.index as usize) {
            #[cfg(debug_assertions)]
            {
                assert_gen_consistency(self, entity);
            }

            slot.map(|idx| &self.components[idx])
        } else {
            None
        }
    }

    #[inline]
    pub fn get_component_mut(&mut self, entity: Entity) -> Option<&mut T> {
        if let Some(slot) = self.entity_comp_index.get(entity.index as usize) {
            #[cfg(debug_assertions)]
            {
                assert_gen_consistency(self, entity);
            }

            slot.map(move |idx| &mut self.components[idx])
        } else {
            None
        }
    }
}

// This trait is required to define common methods for all storages
// so we can iterate on them in an untyped fashion and still be
// able to do stuff on them (e.g. remove_all_components)
trait Component_Storage_Interface: anymap::any::Any {
    fn remove_component(&mut self, entity: Entity);

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
