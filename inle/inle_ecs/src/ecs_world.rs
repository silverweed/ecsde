use super::comp_mgr::{self, Component_Manager};
use inle_alloc::gen_alloc::{Generational_Allocator, Generational_Index};
use inle_events::evt_register;
use std::any::type_name;
use std::collections::HashSet;

pub type Entity = Generational_Index;

pub type Component_Storage<T> = comp_mgr::Component_Storage<T>;
pub type Component_Storage_Read<'a, T> = comp_mgr::Component_Storage_Read<'a, T>;
pub type Component_Storage_Write<'a, T> = comp_mgr::Component_Storage_Write<'a, T>;
pub type Component_Read<'l, T> = comp_mgr::Component_Read<'l, T>;
pub type Component_Write<'l, T> = comp_mgr::Component_Write<'l, T>;

pub struct Evt_Entity_Destroyed;

impl evt_register::Event for Evt_Entity_Destroyed {
    type Args = Entity;
}

pub struct Ecs_World {
    entity_manager: Entity_Manager,

    // Note: must be visible to entity_stream
    pub(super) component_manager: Component_Manager,

    entities_pending_destroy_notify: HashSet<Entity>,
    entities_pending_destroy: Vec<Entity>,
}

impl Ecs_World {
    pub fn new() -> Ecs_World {
        Ecs_World {
            entity_manager: Entity_Manager::new(),
            component_manager: Component_Manager::new(),
            entities_pending_destroy_notify: HashSet::new(),
            entities_pending_destroy: vec![],
        }
    }

    pub fn new_entity(&mut self) -> Entity {
        self.entity_manager.new_entity()
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entity_manager.entities
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        debug_assert!(self.is_valid_entity(entity));
        self.entities_pending_destroy_notify.insert(entity);
    }

    pub fn notify_destroyed(&self, evt_register: &mut evt_register::Event_Register) {
        let data: Vec<&Entity> = self.entities_pending_destroy_notify.iter().collect();
        evt_register.raise_batch::<Evt_Entity_Destroyed>(&data);
    }

    pub fn destroy_pending(&mut self) -> Vec<Entity> {
        for &entity in &self.entities_pending_destroy {
            self.component_manager.remove_all_components(entity);
            self.entity_manager.destroy_entity(entity);
        }
        let destroyed = std::mem::take(&mut self.entities_pending_destroy);
        self.entities_pending_destroy = self.entities_pending_destroy_notify.drain().collect();
        destroyed
    }

    pub fn is_valid_entity(&self, entity: Entity) -> bool {
        self.entity_manager.is_valid_entity(entity)
            && !self.entities_pending_destroy.contains(&entity)
    }

    pub fn add_component<T: 'static>(&mut self, entity: Entity, data: T) {
        self.component_manager.add_component::<T>(entity, data);
    }

    #[inline]
    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<Component_Read<T>> {
        trace!("get_component");

        if !self.entity_manager.is_valid_entity(entity) {
            fatal!(
                "get_component::<{}?>: invalid entity {:?}",
                type_name::<T>(),
                entity
            );
        }

        self.component_manager.get_component(entity)
    }

    #[inline]
    pub fn get_component_mut<T: 'static>(&self, entity: Entity) -> Option<Component_Write<T>> {
        trace!("get_component_mut");

        if !self.entity_manager.is_valid_entity(entity) {
            fatal!(
                "get_component_mut::<{}?>: invalid entity {:?}",
                type_name::<T>(),
                entity
            );
        }

        self.component_manager.get_component_mut(entity)
    }

    pub fn remove_component<T: 'static>(&mut self, entity: Entity) {
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

    #[inline]
    pub fn has_component<T: 'static>(&self, entity: Entity) -> bool {
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

    #[inline]
    pub fn get_component_storage<T: Copy + 'static>(&self) -> Option<&Component_Storage<T>> {
        self.component_manager.get_component_storage::<T>()
    }

    #[inline]
    pub fn get_component_storage_mut<T: Copy + 'static>(
        &mut self,
    ) -> Option<&mut Component_Storage<T>> {
        self.component_manager.get_component_storage_mut::<T>()
    }

    #[inline]
    pub fn read_component_storage<T: 'static>(&self) -> Option<Component_Storage_Read<T>> {
        self.component_manager.read_component_storage::<T>()
    }

    #[inline]
    pub fn write_component_storage<T: 'static>(&self) -> Option<Component_Storage_Write<T>> {
        self.component_manager.write_component_storage::<T>()
    }
}

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
        self.entities.swap_remove(idx);
    }

    pub fn is_valid_entity(&self, entity: Entity) -> bool {
        self.alloc.is_valid(entity)
    }

    pub fn n_live_entities(&self) -> usize {
        self.entities.len()
    }
}

#[cfg(debug_assertions)]
impl Ecs_World {
    pub fn get_comp_name_list_for_entity(&self, entity: Entity) -> Vec<&'static str> {
        self.component_manager.get_comp_name_list_for_entity(entity)
    }
}

//pub trait System {
//fn get_queries_mut(&mut self) -> &mut [crate::ecs_query_new::Ecs_Query];
//}

//pub struct System_Handle(usize);

//struct System_Manager {
//systems: Vec<Box<dyn System>>,
//}

//impl System_Manager {
//fn register_system(&mut self, system: Box<dyn System>) -> System_Handle {
//self.systems.push(system);
//System_Handle(self.systems.len() - 1)
//}
//}

#[cfg(tests)]
include!("./ecs_world_tests.rs");
