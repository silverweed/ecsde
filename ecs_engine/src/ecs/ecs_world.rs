use super::comp_mgr::{self, Component_Manager};
use crate::alloc::gen_alloc::{Generational_Allocator, Generational_Index};
use crate::common::bitset::Bit_Set;
use crate::events::evt_register;
use std::any::type_name;
use std::collections::HashSet;

#[cfg(debug_assertions)]
use crate::debug::painter::Debug_Painter;

// NOTE: we reserve the MSB to distinguish static (0)/dynamic (1)
pub type Entity = Generational_Index;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Entity_Static_Or_Dyn {
    Static,
    Dynamic
}

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

    pub fn new_entity(&mut self, static_or_dyn: Entity_Static_Or_Dyn) -> Entity {
        self.entity_manager.new_entity(static_or_dyn)
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entity_manager.entities
    }

    pub fn static_entities(&self) -> &[Entity] {
        &self.entity_manager.static_entities
    }

    pub fn dynamic_entities(&self) -> &[Entity] {
        &self.entity_manager.dyn_entities
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
        let destroyed = self.entities_pending_destroy.split_off(0);
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

pub struct Entity_Manager {
    alloc: Generational_Allocator,
    // All entities, both static and dynamic
    entities: Vec<Entity>,
    // Entities that cannot ever move
    static_entities: Vec<Entity>,
    // Entities that can move
    dyn_entities: Vec<Entity>,
}

impl Entity_Manager {
    pub fn new() -> Entity_Manager {
        Entity_Manager {
            alloc: Generational_Allocator::new(64),
            entities: vec![],
            static_entities: vec![],
            dyn_entities: vec![],
        }
    }

    pub fn new_entity(&mut self, static_or_dyn: Entity_Static_Or_Dyn) -> Entity {
        let mut e = self.alloc.allocate();
        assert!(e.gen < 0x8000, "Entity generation too high!");

        if static_or_dyn == Entity_Static_Or_Dyn::Static {
            Self::make_static(&mut e);
            self.static_entities.push(e);
        } else {
            Self::make_dynamic(&mut e);
            self.dyn_entities.push(e);
        }
        self.entities.push(e);
        debug_assert_eq!(self.entities.len(), self.static_entities.len() + self.dyn_entities.len());

        e
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.alloc.deallocate(entity);
        // @Cleanup @WaitForStable: use remove_item
        let idx = self.entities.iter().position(|e| *e == entity).unwrap();
        self.entities.remove(idx);
        if Self::is_static(entity) {
            let idx = self.static_entities.iter().position(|e| *e == entity).unwrap();
            self.static_entities.remove(idx);
        } else {
            let idx = self.dyn_entities.iter().position(|e| *e == entity).unwrap();
            self.dyn_entities.remove(idx);
        }
    }

    pub fn is_valid_entity(&self, entity: Entity) -> bool {
        self.alloc.is_valid(Self::to_valid_gen_index(entity))
    }

    pub fn n_live_entities(&self) -> usize {
        self.entities.len()
    }

    fn make_static(entity: &mut Entity) {
        entity.gen &= 0x7FFF;
    }

    fn make_dynamic(entity: &mut Entity) {
        entity.gen |= 0x8000;
    }

    fn is_static(entity: Entity) -> bool {
        entity.gen & 0x8000 == 0
    }

    // Inverse function of make_static/make_dynamic
    fn to_valid_gen_index(entity: Entity) -> Entity {
        Entity {
            gen: entity.gen & 0x7FFF,
            index: entity.index,
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

#[cfg(debug_assertions)]
pub fn draw_comp_alloc<T: 'static + Copy>(world: &Ecs_World, painter: &mut Debug_Painter) {
    comp_mgr::draw_comp_alloc::<T>(world, painter);
}

#[cfg(tests)]
include!("./ecs_world_tests.rs");
