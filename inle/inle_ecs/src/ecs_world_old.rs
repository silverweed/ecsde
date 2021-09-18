use super::comp_mgr_new::{self as comp_mgr, Component_Manager};
use inle_alloc::gen_alloc::{Generational_Allocator, Generational_Index};
use inle_common::bitset::Bit_Set;
use inle_events::evt_register;
use std::any::type_name;
use std::collections::HashSet;
use std::marker::PhantomData;

#[cfg(debug_assertions)]
use inle_debug::painter::Debug_Painter;

pub type Entity = Generational_Index;
pub type Component_Handle = comp_mgr::Component_Handle;

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
        debug_assert!(self.is_valid_entity(entity));
        self.entities_pending_destroy_notify.insert(entity);
    }

    pub fn notify_destroyed(&self, evt_register: &mut evt_register::Event_Register) {
        for &entity in &self.entities_pending_destroy_notify {
            evt_register.raise::<Evt_Entity_Destroyed>(entity);
        }
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

    pub fn register_component<T: 'static + Copy>(&mut self) {
        self.component_manager.register_component::<T>();
    }

    pub fn add_component<T: 'static + Copy>(&mut self, entity: Entity, data: T) -> &mut T {
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

impl<'a, T: 'a + Copy> IntoIterator for &Component_Storage<'a, T> {
    type Item = (Entity, &'a T);
    type IntoIter = Component_Storage_Iterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self.storage)
    }
}

pub struct Component_Storage_Iterator<'a, T> {
    storage: &'a comp_mgr::Component_Storage,
    comp_map_iter: std::collections::hash_map::Iter<'a, Entity, u32>,
    _pd: PhantomData<T>,
}

impl<'a, T: 'a + Copy> Component_Storage_Iterator<'a, T> {
    fn new<'b: 'a>(storage: &'b comp_mgr::Component_Storage) -> Self {
        Self {
            storage,
            comp_map_iter: storage.ent_comp_map.iter(),
            _pd: PhantomData,
        }
    }
}

impl<'a, T: 'a + Copy> Iterator for Component_Storage_Iterator<'a, T> {
    type Item = (Entity, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let (entity, idx) = self.comp_map_iter.next()?;
        // @Cleanup: maybe refactor this to be hidden inside comp_mgr
        let comp = unsafe { self.storage.alloc.get(*idx) };
        Some((*entity, comp))
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

#[cfg(debug_assertions)]
pub fn draw_comp_alloc<T: 'static + Copy>(world: &Ecs_World, painter: &mut Debug_Painter) {
    comp_mgr::draw_comp_alloc::<T>(world, painter);
}

#[cfg(debug_assertions)]
pub fn component_name_from_handle(world: &Ecs_World, handle: Component_Handle) -> Option<&str> {
    let idx = handle as usize;
    let names = &world.component_manager.debug.comp_names;
    if idx < names.len() {
        Some(names[idx])
    } else {
        None
    }
}

#[cfg(tests)]
include!("./ecs_world_tests.rs");
