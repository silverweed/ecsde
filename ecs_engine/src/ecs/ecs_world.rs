use super::comp_mgr::{self, Component_Manager};
use crate::alloc::gen_alloc::{Generational_Allocator, Generational_Index};
use crate::common::bitset::Bit_Set;
use std::any::type_name;
use std::vec::Vec;

pub type Entity = Generational_Index;

pub struct Entity_Manager {
    alloc: Generational_Allocator,
    entities: Vec<Entity>,
}

impl Entity_Manager {
    pub fn new() -> Entity_Manager {
        Entity_Manager {
            alloc: Generational_Allocator::new(64),
            entities: vec![],
        }
    }

    pub fn new_entity(&mut self) -> Entity {
        let e = self.alloc.allocate();
        self.entities.push(e);
        e
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.alloc.deallocate(entity);
        let idx = self.entities.iter().position(|e| *e == entity).unwrap();
        self.entities.remove(idx);
    }

    pub fn is_valid_entity(&self, entity: Entity) -> bool {
        self.alloc.is_valid(entity)
    }

    pub fn n_live_entities(&self) -> usize {
        self.entities.len()
    }
}

pub struct Ecs_World {
    entity_manager: Entity_Manager,
    // Note: must be visible to entity_stream
    pub(super) component_manager: Component_Manager,
}

impl Ecs_World {
    pub fn new() -> Ecs_World {
        Ecs_World {
            entity_manager: Entity_Manager::new(),
            component_manager: Component_Manager::new(),
        }
    }

    pub fn get_entity_comp_set(&self, entity: Entity) -> std::borrow::Cow<'_, Bit_Set> {
        assert!(
            self.entity_manager.is_valid_entity(entity),
            "Invalid entity {:?}",
            entity
        );
        self.component_manager.get_entity_comp_set(entity)
    }

    pub fn new_entity(&mut self) -> Entity {
        self.entity_manager.new_entity()
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entity_manager.entities
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.entity_manager.destroy_entity(entity)
    }

    pub fn is_valid_entity(&self, entity: Entity) -> bool {
        self.entity_manager.is_valid_entity(entity)
    }

    pub fn register_component<T: 'static + Copy>(&mut self) {
        self.component_manager.register_component::<T>();
    }

    pub fn add_component<T: 'static + Copy>(&mut self, entity: Entity, data: T) -> &mut T {
        // @Temporary
        self.component_manager.add_component::<T>(entity, data)
    }

    pub fn get_component<T: 'static + Copy>(&self, entity: Entity) -> Option<&T> {
        trace!("get_component");

        if !self.entity_manager.is_valid_entity(entity) {
            fatal!(
                "get_component::<{}?>: invalid entity {:?}",
                type_name::<T>(),
                entity
            );
        }

        self.component_manager.get_component::<T>(entity)
    }

    pub fn get_component_mut<T: 'static + Copy>(&mut self, entity: Entity) -> Option<&mut T> {
        trace!("get_component_mut");

        if !self.entity_manager.is_valid_entity(entity) {
            fatal!(
                "get_component_mut::<{}?>: invalid entity {:?}",
                type_name::<T>(),
                entity
            );
        }

        self.component_manager.get_component_mut::<T>(entity)
    }

    pub fn remove_component<T: 'static + Copy>(&mut self, entity: Entity) {
        trace!("remove_component");

        if !self.entity_manager.is_valid_entity(entity) {
            fatal!(
                "remove_component::<{}?>: invalid entity {:?}",
                type_name::<T>(),
                entity
            );
        }

        self.component_manager.remove_component::<T>(entity);
    }

    pub fn has_component<T: 'static + Copy>(&self, entity: Entity) -> bool {
        trace!("has_component");

        if !self.entity_manager.is_valid_entity(entity) {
            fatal!(
                "has_component::<{}?>: invalid entity {:?}",
                type_name::<T>(),
                entity
            );
        }

        self.component_manager.has_component::<T>(entity)
    }

    pub fn get_components<T: 'static + Copy>(&self) -> impl Iterator<Item = &T> {
        trace!("get_components");

        self.component_manager.get_components::<T>()
    }

    pub fn get_components_mut<T: 'static + Copy>(&mut self) -> impl Iterator<Item = &mut T> {
        trace!("get_components_mut");

        self.component_manager.get_components_mut::<T>()
    }

    pub fn get_component_storage<T: Copy + 'static>(&self) -> Component_Storage<T> {
        Component_Storage {
            storage: self.component_manager.get_component_storage::<T>(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn get_component_storage_mut<T: Copy + 'static>(&mut self) -> Component_Storage_Mut<T> {
        Component_Storage_Mut {
            storage: self.component_manager.get_component_storage_mut::<T>(),
            _marker: std::marker::PhantomData,
        }
    }
}

// Conveniency wrapper for the untyped Component_Storage
pub struct Component_Storage<'a, T> {
    storage: &'a comp_mgr::Component_Storage,
    _marker: std::marker::PhantomData<T>,
}

pub struct Component_Storage_Mut<'a, T> {
    storage: &'a mut comp_mgr::Component_Storage,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Copy> Component_Storage<'_, T> {
    pub fn has_component(&self, entity: Entity) -> bool {
        self.storage.has_component::<T>(entity)
    }

    pub fn get_component(&self, entity: Entity) -> Option<&T> {
        self.storage.get_component::<T>(entity)
    }
}

impl<T: Copy> Component_Storage_Mut<'_, T> {
    pub fn has_component(&self, entity: Entity) -> bool {
        self.storage.has_component::<T>(entity)
    }

    pub fn get_component(&self, entity: Entity) -> Option<&T> {
        self.storage.get_component::<T>(entity)
    }

    pub fn get_component_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.storage.get_component_mut::<T>(entity)
    }
}

#[cfg(tests)]
include!("./ecs_world_tests.rs");
