use ecs_engine::alloc::generational_allocator::{
    Gen_Type, Generational_Allocator, Generational_Index, Index_Type,
};
use ecs_engine::core::common::bitset::Bit_Set;
use std::any::{type_name, TypeId};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::vec::Vec;

pub type Entity = Generational_Index;
pub type Entity_Index = usize;
pub type Component_Handle = u32;

#[derive(Default, Clone)]
struct Component_Storage {
    pub individual_size: usize,
    pub data: Vec<u8>,
    // { entity => index into `data` }
    pub comp_idx: HashMap<Entity_Index, usize>,
}

pub struct Component_Manager {
    components: Vec<Component_Storage>,
    last_comp_handle: Component_Handle,
    // Indexed by entity index
    pub(super) entity_comp_set: Vec<Bit_Set>,
}

impl Component_Manager {
    pub fn new() -> Component_Manager {
        Component_Manager {
            components: vec![],
            last_comp_handle: 0,
            entity_comp_set: vec![],
        }
    }

    pub fn register_component<T>(&mut self) -> Component_Handle {
        let handle = self.last_comp_handle;

        self.components
            .resize(handle as usize + 1, Component_Storage::default());
        self.components[handle as usize].individual_size = std::mem::size_of::<T>();

        self.last_comp_handle += 1;

        handle
    }

    pub fn add_component(&mut self, entity: Entity, comp_handle: Component_Handle) -> *mut u8 {
        let storage = self
            .components
            .get_mut(comp_handle as usize)
            .unwrap_or_else(|| panic!("Invalid component handle {:?}", comp_handle));

        let individual_size = storage.individual_size;
        let index = if let Some(index) = storage.comp_idx.get(&entity.index) {
            *index
        } else {
            let n_comps = storage.data.len() / individual_size;
            storage.data.resize(storage.data.len() + individual_size, 0);
            storage.comp_idx.insert(entity.index, n_comps);
            self.entity_comp_set
                .resize(entity.index + 1, Bit_Set::default());
            self.entity_comp_set[entity.index].set(comp_handle as usize, true);
            n_comps
        };

        unsafe { storage.data.as_mut_ptr().add(index * individual_size) }
    }

    pub fn get_component(
        &self,
        entity: Entity,
        comp_handle: Component_Handle,
    ) -> Option<*const u8> {
        let storage = &self.components[comp_handle as usize];
        if let Some(index) = storage.comp_idx.get(&entity.index) {
            let comp = unsafe { storage.data.as_ptr().add(*index * storage.individual_size) };
            Some(comp)
        } else {
            None
        }
    }

    pub fn get_component_mut(
        &mut self,
        entity: Entity,
        comp_handle: Component_Handle,
    ) -> Option<*mut u8> {
        let storage = self
            .components
            .get_mut(comp_handle as usize)
            .unwrap_or_else(|| panic!("Invalid component handle {:?}", comp_handle));
        if let Some(index) = storage.comp_idx.get_mut(&entity.index) {
            let comp = unsafe {
                storage
                    .data
                    .as_mut_ptr()
                    .add(*index * storage.individual_size)
            };
            Some(comp)
        } else {
            None
        }
    }

    pub fn remove_component(&mut self, entity: Entity, comp_handle: Component_Handle) {
        self.entity_comp_set[entity.index].set(comp_handle as usize, false);
        self.components[comp_handle as usize]
            .comp_idx
            .remove(&entity.index);
    }

    pub fn has_component(&self, entity: Entity, comp_handle: Component_Handle) -> bool {
        self.entity_comp_set[entity.index].get(comp_handle as usize)
    }
}

pub struct Entity_Manager {
    alloc: Generational_Allocator,
}

impl Entity_Manager {
    pub fn new() -> Entity_Manager {
        Entity_Manager {
            alloc: Generational_Allocator::new(64),
        }
    }

    pub fn new_entity(&mut self) -> Entity {
        self.alloc.allocate()
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.alloc.deallocate(entity);
    }

    pub fn is_valid_entity(&self, entity: Entity) -> bool {
        self.alloc.is_valid(entity)
    }

    pub(super) fn cur_gen(&self, idx: Index_Type) -> Gen_Type {
        self.alloc.cur_gen(idx)
    }
}

pub struct Ecs_World {
    pub component_handles: HashMap<TypeId, Component_Handle>,
    pub entity_manager: Entity_Manager,
    pub component_manager: Component_Manager,
}

impl Ecs_World {
    pub fn new() -> Ecs_World {
        Ecs_World {
            component_handles: HashMap::new(),
            entity_manager: Entity_Manager::new(),
            component_manager: Component_Manager::new(),
        }
    }

    pub fn new_entity(&mut self) -> Entity {
        self.entity_manager.new_entity()
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.entity_manager.destroy_entity(entity)
    }

    pub fn is_valid_entity(&self, entity: Entity) -> bool {
        self.entity_manager.is_valid_entity(entity)
    }

    pub fn register_component<T: 'static + Copy>(&mut self) {
        let type_id = TypeId::of::<T>();
        match self.component_handles.entry(type_id) {
            Entry::Occupied(_) => {
                panic!(
                    "register_component: same component '{}' registered twice!",
                    type_name::<T>()
                );
            }
            Entry::Vacant(v) => {
                let handle = self.component_manager.register_component::<T>();
                v.insert(handle);
            }
        }
    }

    pub fn add_component<T: 'static + Copy>(&mut self, entity: Entity) -> &mut T {
        if !self.entity_manager.is_valid_entity(entity) {
            panic!(
                "add_component::<{}?>: invalid entity {:?}",
                type_name::<T>(),
                entity
            );
        }
        let handle = self.component_handles.get(&TypeId::of::<T>()).unwrap();
        unsafe { std::mem::transmute(self.component_manager.add_component(entity, *handle)) }
    }

    pub fn get_component<T: 'static + Copy>(&self, entity: Entity) -> Option<&T> {
        if !self.entity_manager.is_valid_entity(entity) {
            panic!(
                "get_component::<{}?>: invalid entity {:?}",
                type_name::<T>(),
                entity
            );
        }
        let handle = self.component_handles.get(&TypeId::of::<T>()).unwrap();
        self.component_manager
            .get_component(entity, *handle)
            // reinterpret cast from *const u8 to *const T
            .map(|ptr| unsafe { &*(ptr as *const T) })
    }

    pub fn get_component_mut<T: 'static + Copy>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.entity_manager.is_valid_entity(entity) {
            panic!(
                "get_component_mut::<{}?>: invalid entity {:?}",
                type_name::<T>(),
                entity
            );
        }
        let handle = self.component_handles.get(&TypeId::of::<T>()).unwrap();
        self.component_manager
            .get_component_mut(entity, *handle)
            // reinterpret cast from *mut u8 to *mut T
            .map(|ptr| unsafe { &mut *(ptr as *mut T) })
    }

    pub fn remove_component<T: 'static + Copy>(&mut self, entity: Entity) {
        if !self.entity_manager.is_valid_entity(entity) {
            panic!(
                "remove_component::<{}?>: invalid entity {:?}",
                type_name::<T>(),
                entity
            );
        }
        let handle = self.component_handles.get(&TypeId::of::<T>()).unwrap();
        self.component_manager.remove_component(entity, *handle);
    }

    pub fn has_component<T: 'static + Copy>(&self, entity: Entity) -> bool {
        if !self.entity_manager.is_valid_entity(entity) {
            panic!(
                "has_component::<{}?>: invalid entity {:?}",
                type_name::<T>(),
                entity
            );
        }
        let handle = self.component_handles.get(&TypeId::of::<T>()).unwrap();
        self.component_manager.has_component(entity, *handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Debug, Default)]
    struct C_Test {
        foo: i32,
    }

    #[derive(Copy, Clone, Debug, Default)]
    struct C_Test2 {
        foo: i32,
    }

    #[derive(Copy, Clone, Debug, Default)]
    struct C_Test3 {
        foo: i32,
    }

    #[test]
    #[should_panic]
    fn register_same_component_twice() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        em.register_component::<C_Test>();
    }

    #[test]
    fn get_component() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();

        let e = em.new_entity();
        assert!(em.get_component::<C_Test>(e).is_none());

        em.add_component::<C_Test>(e);
        assert!(em.get_component::<C_Test>(e).is_some());
    }

    #[test]
    fn get_component_mut() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();

        let e = em.new_entity();
        assert!(em.get_component_mut::<C_Test>(e).is_none());

        em.add_component::<C_Test>(e);
        assert!(em.get_component_mut::<C_Test>(e).is_some());
    }

    #[test]
    fn mutate_component() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();

        let e = em.new_entity();
        {
            let mut c = em.add_component::<C_Test>(e);
            c.foo = 4242;
        }

        assert!(em.get_component::<C_Test>(e).unwrap().foo == 4242);
    }

    #[test]
    #[should_panic]
    fn add_component_inexisting_entity() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        em.add_component::<C_Test>(Entity { index: 0, gen: 1 });
    }

    #[test]
    #[should_panic]
    fn get_component_inexisting_entity() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        em.get_component::<C_Test>(Entity { index: 0, gen: 1 });
    }

    #[test]
    #[should_panic]
    fn get_component_mut_inexisting_entity() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        em.get_component_mut::<C_Test>(Entity { index: 0, gen: 1 });
    }

    #[test]
    fn destroy_entity() {
        let mut em = Ecs_World::new();
        let e = em.new_entity();
        em.destroy_entity(e);
        assert!(!em.is_valid_entity(e));
    }

    #[test]
    #[should_panic]
    fn double_free_entity() {
        let mut em = Ecs_World::new();
        let e = em.new_entity();
        em.destroy_entity(e);
        em.destroy_entity(e);
    }

    #[test]
    #[should_panic]
    fn destroy_inexisting_entity() {
        let mut em = Ecs_World::new();
        em.destroy_entity(Entity { index: 0, gen: 1 });
    }

    #[test]
    #[should_panic]
    fn add_component_destroyed_entity() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.destroy_entity(e);
        em.add_component::<C_Test>(e);
    }

    #[test]
    #[should_panic]
    fn get_component_destroyed_entity() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.destroy_entity(e);
        em.get_component::<C_Test>(e);
    }

    #[test]
    #[should_panic]
    fn get_component_destroyed_and_recreated_entity() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.destroy_entity(e);
        em.new_entity();
        em.get_component::<C_Test>(e);
    }

    #[test]
    fn get_component_destroyed_and_recreated_entity_good() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();

        let e1 = em.new_entity();
        em.add_component::<C_Test>(e1);
        em.destroy_entity(e1);

        let e2 = em.new_entity();
        em.get_component::<C_Test>(e2);
    }

    #[test]
    fn remove_component() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.remove_component::<C_Test>(e);
    }

    #[test]
    fn double_remove_component() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.remove_component::<C_Test>(e);
        em.remove_component::<C_Test>(e);
    }

    #[test]
    fn get_removed_component() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.remove_component::<C_Test>(e);
        assert!(em.get_component::<C_Test>(e).is_none());
    }

    #[test]
    fn remove_and_readd_component() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.remove_component::<C_Test>(e);
        em.add_component::<C_Test>(e);
        em.get_component::<C_Test>(e);
    }

    #[test]
    #[should_panic]
    fn remove_component_destroyed_and_recreated_entity() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.destroy_entity(e);
        em.new_entity();
        em.remove_component::<C_Test>(e);
    }

    #[test]
    fn has_get_consistency() {
        let mut em = Ecs_World::new();
        let mut entities: Vec<Entity> = vec![];
        em.register_component::<C_Test>();
        em.register_component::<C_Test2>();
        for i in 0..100 {
            let e = em.new_entity();
            entities.push(e);
            em.add_component::<C_Test>(e);
            if i % 2 == 0 {
                em.add_component::<C_Test2>(e);
            }
        }

        {
            let filtered: Vec<Entity> = entities
                .iter()
                .filter(|&&e| em.has_component::<C_Test>(e) && em.has_component::<C_Test2>(e))
                .cloned()
                .collect();
            for e in filtered {
                assert!(em.get_component::<C_Test>(e).is_some());
                assert!(em.get_component::<C_Test2>(e).is_some());
            }
        }
        {
            let filtered: Vec<Entity> = entities
                .iter()
                .filter(|&&e| em.has_component::<C_Test>(e))
                .cloned()
                .collect();
            for e in filtered {
                assert!(em.get_component::<C_Test>(e).is_some());
            }
        }
        {
            let filtered: Vec<Entity> = entities
                .iter()
                .filter(|&&e| em.has_component::<C_Test>(e) && !em.has_component::<C_Test2>(e))
                .cloned()
                .collect();
            for e in filtered {
                assert!(em.get_component::<C_Test>(e).is_some());
                assert!(em.get_component::<C_Test2>(e).is_none());
            }
        }
    }
}
