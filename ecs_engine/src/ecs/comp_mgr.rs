mod comp_alloc;

use super::ecs_world::Entity;
use crate::common::bitset::Bit_Set;
use comp_alloc::Component_Allocator;
use std::any::{type_name, TypeId};
use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem::size_of;

#[cfg(debug_assertions)]
use crate::debug::painter::Debug_Painter;

// Note: must be visible to entity_stream
pub(super) type Component_Handle = u32;

pub struct Component_Manager {
    /// Storage for all non-ZST components
    storages: Vec<Component_Storage>,

    last_comp_handle: Component_Handle,
    handles: HashMap<TypeId, Component_Handle>,

    /// Indexed by entity index
    entity_comp_set: Vec<Bit_Set>,
}

pub struct Component_Storage {
    alloc: Component_Allocator,
    ent_comp_map: HashMap<Entity, u32>,
    comp_layout: std::alloc::Layout,
}

impl Component_Storage {
    pub fn new<T: Copy>() -> Self {
        Self {
            alloc: Component_Allocator::new::<T>(),
            ent_comp_map: HashMap::new(),
            comp_layout: Component_Allocator::get_comp_layout::<T>(),
        }
    }

    pub fn has_component<T>(&self, entity: Entity) -> bool {
        self.ent_comp_map.contains_key(&entity)
    }

    pub fn get_component<T: Copy>(&self, entity: Entity) -> Option<&T> {
        self.ent_comp_map
            .get(&entity)
            // Note: safe as long as ent_comp_map is in sync with the allocator
            .map(|&idx| unsafe { self.alloc.get(idx) })
    }

    pub fn get_component_mut<T: Copy>(&mut self, entity: Entity) -> Option<&mut T> {
        self.ent_comp_map
            .get(&entity)
            .cloned()
            // Note: safe as long as ent_comp_map is in sync with the allocator
            .map(move |idx| unsafe { self.alloc.get_mut(idx) })
    }

    pub fn add_component<T: Copy>(&mut self, entity: Entity, data: T) -> &mut T {
        match self.ent_comp_map.entry(entity) {
            Entry::Occupied(_) => {
                fatal!(
                    "Entity {:?} already has component {:?}!",
                    entity,
                    type_name::<T>()
                );
            }
            Entry::Vacant(v) => {
                let (idx, comp) = self.alloc.add(data);
                v.insert(idx);
                comp
            }
        }
    }

    pub fn remove_component<T: Copy>(&mut self, entity: Entity) {
        let idx = self.ent_comp_map.get(&entity).unwrap_or_else(|| {
            fatal!(
                "Tried to remove inexisting component {:?} from entity {:?}",
                type_name::<T>(),
                entity
            )
        });
        // Note: safe as long as ent_comp_map is in sync with the allocator
        unsafe {
            self.alloc.remove::<T>(*idx);
        }
    }

    pub fn remove_component_dyn(&mut self, entity: Entity) {
        let idx = self.ent_comp_map.get(&entity).unwrap_or_else(|| {
            fatal!(
                "Tried to remove inexisting component from entity {:?}",
                entity
            )
        });
        // Note: safe as long as ent_comp_map is in sync with the allocator
        unsafe {
            self.alloc.remove_dyn(*idx, &self.comp_layout);
        }
    }
}

impl Component_Manager {
    pub fn new() -> Self {
        Self {
            storages: vec![],
            last_comp_handle: 0,
            handles: HashMap::new(),
            entity_comp_set: vec![],
        }
    }

    pub fn register_component<T: Copy + 'static>(&mut self) -> Component_Handle {
        let comp_id = TypeId::of::<T>();
        let handles_entry = match self.handles.entry(comp_id) {
            Entry::Occupied(_) => {
                fatal!("Component {:?} registered twice!", type_name::<T>());
            }
            Entry::Vacant(v) => v,
        };

        let handle = self.last_comp_handle;
        if size_of::<T>() != 0 {
            self.storages.push(Component_Storage::new::<T>());
            self.last_comp_handle += 1;
        }

        handles_entry.insert(handle);

        handle
    }

    pub fn has_component<T: 'static>(&self, entity: Entity) -> bool {
        let handle = self.get_handle::<T>();
        let bit_is_set = self.entity_comp_set[entity.index as usize].get(handle as usize);

        #[cfg(debug_assertions)]
        {
            if size_of::<T>() != 0 {
                debug_assert_eq!(
                    self.storages[handle as usize].has_component::<T>(entity),
                    bit_is_set
                );
            }
        }

        bit_is_set
    }

    pub fn get_component<T: Copy + 'static>(&self, entity: Entity) -> Option<&T> {
        static UNIT: () = ();

        let handle = self.get_handle::<T>();
        if size_of::<T>() == 0 {
            if self.has_component::<T>(entity) {
                unsafe { Some(&*(&UNIT as *const () as *const T)) }
            } else {
                None
            }
        } else {
            self.storages[handle as usize].get_component::<T>(entity)
        }
    }

    pub fn get_component_mut<T: Copy + 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        static mut UNIT: () = ();

        let handle = self.get_handle::<T>();
        if size_of::<T>() == 0 {
            if self.has_component::<T>(entity) {
                unsafe { Some(&mut *(&mut UNIT as *mut () as *mut T)) }
            } else {
                None
            }
        } else {
            self.storages[handle as usize].get_component_mut::<T>(entity)
        }
    }

    pub fn add_component<T: Copy + 'static>(&mut self, entity: Entity, data: T) -> &mut T {
        if self.entity_comp_set.len() <= entity.index as usize {
            self.entity_comp_set
                .resize(entity.index as usize + 1, Bit_Set::default());
        }
        let handle = self.get_handle::<T>();
        self.entity_comp_set[entity.index as usize].set(handle as usize, true);

        if size_of::<T>() != 0 {
            self.storages[handle as usize].add_component::<T>(entity, data)
        } else {
            static mut UNIT: () = ();
            unsafe { &mut *(&mut UNIT as *mut () as *mut T) }
        }
    }

    pub fn remove_component<T: Copy + 'static>(&mut self, entity: Entity) {
        let handle = self.get_handle::<T>();
        self.entity_comp_set[entity.index as usize].set(handle as usize, false);
        self.storages[handle as usize].remove_component::<T>(entity);
    }

    pub fn remove_all_components(&mut self, entity: Entity) {
        let comp_set = &self.entity_comp_set[entity.index as usize];
        for handle in comp_set {
            self.storages[handle as usize].remove_component_dyn(entity);
        }
    }

    pub fn get_components<T: Copy + 'static>(&self) -> impl Iterator<Item = &T> {
        if size_of::<T>() == 0 {
            comp_alloc::Component_Allocator_Iter::empty()
        } else {
            let handle = self.get_handle::<T>();
            self.storages[handle as usize].alloc.iter::<T>()
        }
    }

    pub fn get_components_mut<T: Copy + 'static>(&mut self) -> impl Iterator<Item = &mut T> {
        if size_of::<T>() == 0 {
            comp_alloc::Component_Allocator_Iter_Mut::empty()
        } else {
            let handle = self.get_handle::<T>();
            self.storages[handle as usize].alloc.iter_mut::<T>()
        }
    }

    pub fn get_component_storage<T: Copy + 'static>(&self) -> &Component_Storage {
        assert_ne!(
            size_of::<T>(),
            0,
            "Cannot get storage of Component {:?} (has zero size)",
            type_name::<T>()
        );
        let handle = self.get_handle::<T>();
        &self.storages[handle as usize]
    }

    pub fn get_component_storage_mut<T: Copy + 'static>(&mut self) -> &mut Component_Storage {
        assert_ne!(
            size_of::<T>(),
            0,
            "Cannot get storage of Component {:?} (has zero size)",
            type_name::<T>()
        );
        let handle = self.get_handle::<T>();
        &mut self.storages[handle as usize]
    }

    pub fn get_entity_comp_set(&self, entity: Entity) -> Cow<'_, Bit_Set> {
        if (entity.index as usize) < self.entity_comp_set.len() {
            Cow::Borrowed(&self.entity_comp_set[entity.index as usize])
        } else {
            Cow::Owned(Bit_Set::default())
        }
    }

    #[inline(always)]
    // Note: must be visible to entity_stream
    pub(super) fn get_handle<T: 'static>(&self) -> Component_Handle {
        *self
            .handles
            .get(&TypeId::of::<T>())
            .unwrap_or_else(|| fatal!("Component {:?} was not registered!", type_name::<T>()))
    }
}

#[cfg(debug_assertions)]
pub(super) fn draw_comp_alloc<T: 'static + Copy>(
    world: &super::ecs_world::Ecs_World,
    painter: &mut Debug_Painter,
) {
    world.component_manager.storages[world.component_manager.get_handle::<T>() as usize]
        .alloc
        .debug_draw::<T>(painter);
}
