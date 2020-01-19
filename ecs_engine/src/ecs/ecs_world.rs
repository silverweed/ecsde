use crate::alloc::generational_allocator::{
    Gen_Type, Generational_Allocator, Generational_Index, Index_Type,
};
use crate::core::common::bitset::Bit_Set;
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

#[derive(Clone)]
pub struct Components_Map_Safe<'a, T> {
    comp_storage: &'a Component_Storage,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T> Components_Map_Safe<'a, T> {
    fn new(comp_storage: &'a Component_Storage) -> Self {
        Self {
            comp_storage,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn get_component<'b>(&'b self, entity: Entity) -> Option<&'b T>
    where
        'a: 'b,
    {
        let storage = &*self.comp_storage;
        if let Some(index) = storage.comp_idx.get(&entity.index) {
            let comp = unsafe { storage.data.as_ptr().add(*index * storage.individual_size) };
            Some(unsafe { &*(comp as *const T) })
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Components_Map_Unsafe<T> {
    comp_storage: *mut Component_Storage,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Components_Map_Unsafe<T> {
    fn new(comp_storage: &mut Component_Storage) -> Self {
        Self {
            comp_storage: comp_storage as *mut _,
            _marker: std::marker::PhantomData,
        }
    }

    /// # Safety
    /// The Components_Map_Mut must never outlive the Ecs_World, and at most one Components_Map_Mut
    /// per type is allowed to exist at the same time.
    /// The suggested usage of a Components_Map_Mut is to limit its existence to a single function.
    pub unsafe fn get_component(&self, entity: Entity) -> Option<&T> {
        let storage = &*self.comp_storage;
        if let Some(index) = storage.comp_idx.get(&entity.index) {
            let comp = storage.data.as_ptr().add(*index * storage.individual_size);
            Some(&*(comp as *const T))
        } else {
            None
        }
    }

    /// # Safety
    /// The Components_Map_Mut must never outlive the Ecs_World, and at most one Components_Map_Mut
    /// per type is allowed to exist at the same time.
    /// The suggested usage of a Components_Map_Mut is to limit its existence to a single function.
    pub unsafe fn get_component_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let storage = &*self.comp_storage;
        if let Some(index) = storage.comp_idx.get(&entity.index) {
            let comp = storage.data.as_ptr().add(*index * storage.individual_size);
            Some(&mut *(comp as *mut T))
        } else {
            None
        }
    }
}

pub(super) struct Component_Manager {
    components: Vec<Component_Storage>,
    last_comp_handle: Component_Handle,
    // Indexed by entity index
    pub(super) entity_comp_set: Vec<Bit_Set>,
}

impl Component_Manager {
    fn new() -> Component_Manager {
        Component_Manager {
            components: vec![],
            last_comp_handle: 0,
            entity_comp_set: vec![],
        }
    }

    fn register_component<T>(&mut self) -> Component_Handle {
        let handle = self.last_comp_handle;

        self.components
            .resize(handle as usize + 1, Component_Storage::default());
        self.components[handle as usize].individual_size = std::mem::size_of::<T>();

        self.last_comp_handle += 1;

        handle
    }

    fn add_component(&mut self, entity: Entity, comp_handle: Component_Handle) -> *mut u8 {
        let storage = self
            .components
            .get_mut(comp_handle as usize)
            .unwrap_or_else(|| panic!("Invalid component handle {:?}", comp_handle));

        let individual_size = storage.individual_size;
        let index = if let Some(index) = storage.comp_idx.get(&entity.index) {
            *index
        } else {
            self.entity_comp_set
                .resize(entity.index + 1, Bit_Set::default());
            self.entity_comp_set[entity.index].set(comp_handle as usize, true);

            if individual_size != 0 {
                let n_comps = storage.data.len() / individual_size;
                storage.data.resize(storage.data.len() + individual_size, 0);
                storage.comp_idx.insert(entity.index, n_comps);
                n_comps
            } else {
                // Component is a Zero-Sized Type and doesn't carry any data.
                0
            }
        };

        unsafe { storage.data.as_mut_ptr().add(index * individual_size) }
    }

    fn get_component(&self, entity: Entity, comp_handle: Component_Handle) -> Option<*const u8> {
        let storage = &self.components[comp_handle as usize];

        if storage.individual_size == 0 {
            // ZST component (basically a tag): just check if the component is in the bitset.
            if entity.index < self.entity_comp_set.len()
                && self.entity_comp_set[entity.index].get(comp_handle as usize)
            {
                // return Some(null) to distinguish from the None case.
                Some(std::ptr::null())
            } else {
                None
            }
        } else if let Some(index) = storage.comp_idx.get(&entity.index) {
            let comp = unsafe { storage.data.as_ptr().add(*index * storage.individual_size) };
            Some(comp)
        } else {
            None
        }
    }

    fn get_component_mut(
        &mut self,
        entity: Entity,
        comp_handle: Component_Handle,
    ) -> Option<*mut u8> {
        let storage = self
            .components
            .get_mut(comp_handle as usize)
            .unwrap_or_else(|| panic!("Invalid component handle {:?}", comp_handle));
        if storage.individual_size == 0 {
            // ZST component (basically a tag): just check if the component is in the bitset.
            if self.entity_comp_set[entity.index].get(comp_handle as usize) {
                // return Some(null) to distinguish from the None case.
                Some(std::ptr::null_mut())
            } else {
                None
            }
        } else if let Some(index) = storage.comp_idx.get_mut(&entity.index) {
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

    fn remove_component(&mut self, entity: Entity, comp_handle: Component_Handle) {
        self.entity_comp_set[entity.index].set(comp_handle as usize, false);
        self.components[comp_handle as usize]
            .comp_idx
            .remove(&entity.index);
    }

    fn has_component(&self, entity: Entity, comp_handle: Component_Handle) -> bool {
        self.entity_comp_set[entity.index].get(comp_handle as usize)
    }

    fn get_components(&self, comp_handle: Component_Handle) -> &[u8] {
        &self.components[comp_handle as usize].data
    }

    fn get_components_mut(&mut self, comp_handle: Component_Handle) -> &mut [u8] {
        &mut self.components[comp_handle as usize].data
    }

    fn get_components_storage(&self, comp_handle: Component_Handle) -> &Component_Storage {
        &self.components[comp_handle as usize]
    }

    fn get_components_storage_mut(
        &mut self,
        comp_handle: Component_Handle,
    ) -> &mut Component_Storage {
        &mut self.components[comp_handle as usize]
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

    pub fn n_live_entities(&self) -> usize {
        self.alloc.live_size()
    }
}

pub struct Ecs_World {
    pub component_handles: HashMap<TypeId, Component_Handle>,
    pub entity_manager: Entity_Manager,
    pub(super) component_manager: Component_Manager,
}

macro_rules! get_comp_handle {
    ($handles: expr, $T: ty, $err: expr) => {
        $handles
            .get(&TypeId::of::<$T>())
            .unwrap_or_else(|| panic!($err, type_name::<$T>()))
    };
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

    pub fn add_component<T: 'static + Copy + Default>(&mut self, entity: Entity) -> &mut T {
        if !self.entity_manager.is_valid_entity(entity) {
            panic!(
                "add_component::<{}?>: invalid entity {:?}",
                type_name::<T>(),
                entity
            );
        }
        let handle = get_comp_handle!(
            self.component_handles,
            T,
            "Tried to add inexisting component {:?}!"
        );
        let t = unsafe { &mut *(self.component_manager.add_component(entity, *handle) as *mut T) };
        *t = T::default();
        t
    }

    pub fn get_component<T: 'static + Copy>(&self, entity: Entity) -> Option<&T> {
        if !self.entity_manager.is_valid_entity(entity) {
            panic!(
                "get_component::<{}?>: invalid entity {:?}",
                type_name::<T>(),
                entity
            );
        }
        let handle = get_comp_handle!(
            self.component_handles,
            T,
            "Tried to get unregistered component {:?}!"
        );
        let maybe_comp = self.component_manager.get_component(entity, *handle);
        if std::mem::size_of::<T>() != 0 {
            // reinterpret cast from *const u8 to *const T
            maybe_comp.map(|ptr| unsafe { &*(ptr as *const T) })
        } else {
            maybe_comp.map(|_| unsafe { &*(&() as *const () as *const T) })
        }
    }

    pub fn get_component_mut<T: 'static + Copy>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.entity_manager.is_valid_entity(entity) {
            panic!(
                "get_component_mut::<{}?>: invalid entity {:?}",
                type_name::<T>(),
                entity
            );
        }
        let handle = get_comp_handle!(
            self.component_handles,
            T,
            "Tried to get_mut unregistered component {:?}!"
        );
        let maybe_comp = self.component_manager.get_component_mut(entity, *handle);
        if std::mem::size_of::<T>() != 0 {
            // reinterpret cast from *mut u8 to *mut T
            maybe_comp.map(|ptr| unsafe { &mut *(ptr as *mut T) })
        } else {
            maybe_comp.map(|_| unsafe { &mut *(&mut () as *mut () as *mut T) })
        }
    }

    pub fn remove_component<T: 'static + Copy>(&mut self, entity: Entity) {
        if !self.entity_manager.is_valid_entity(entity) {
            panic!(
                "remove_component::<{}?>: invalid entity {:?}",
                type_name::<T>(),
                entity
            );
        }
        let handle = get_comp_handle!(
            self.component_handles,
            T,
            "Tried to remove unregistered component {:?}!"
        );
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
        let handle = get_comp_handle!(
            self.component_handles,
            T,
            "Tried to test 'has' on unregistered component {:?}!"
        );
        self.component_manager.has_component(entity, *handle)
    }

    pub fn get_components<T: 'static + Copy>(&self) -> &[T] {
        // Note: this should be able to be const, but a pesky compiler error
        // prevents it. Investigate on this later.
        let comp_size = std::mem::size_of::<T>();
        if comp_size == 0 {
            return &[];
        }
        let handle = get_comp_handle!(
            self.component_handles,
            T,
            "Tried to get_all unregistered components {:?}!"
        );
        let data = self.component_manager.get_components(*handle);
        let n_elems = data.len() / comp_size;
        unsafe { std::slice::from_raw_parts(data.as_ptr() as *const T, n_elems) }
    }

    pub fn get_components_mut<T: 'static + Copy>(&mut self) -> &mut [T] {
        let comp_size = std::mem::size_of::<T>();
        if comp_size == 0 {
            return &mut [];
        }
        let handle = get_comp_handle!(
            self.component_handles,
            T,
            "Tried to get_all_mut unregistered components {:?}!"
        );
        let data = self.component_manager.get_components_mut(*handle);
        let n_elems = data.len() / comp_size;
        unsafe { std::slice::from_raw_parts_mut(data.as_ptr() as *mut T, n_elems) }
    }

    /// This is an utility function used when `get_component` is used repeatedly on many entities.
    /// Rather than calling `ecs_world.get_component(entity)` directly, it is more efficient to
    /// first retrieve the components map and then query it (this avoids one map lookup per entity):
    /// ```
    /// # use ecs_engine::ecs::ecs_world::*;
    /// # let mut ecs_world = Ecs_World::new();
    /// # #[derive(Copy, Clone, Default)]
    /// # #[allow(non_camel_case_types)]
    /// # struct My_Comp { _foo: u32 }
    /// # ecs_world.register_component::<My_Comp>();
    /// let comp_map = ecs_world.get_components_map::<My_Comp>();
    /// # let entities: Vec<Entity> = vec![];
    /// for entity in entities {
    ///     let my_comp = comp_map.get_component(entity);
    ///     // ...
    /// }
    /// ```
    pub fn get_components_map<T: 'static + Copy>(&self) -> Components_Map_Safe<T> {
        let comp_size = std::mem::size_of::<T>();
        if comp_size == 0 {
            panic!(
                "[ ERROR ] get_components_map_mut cannot be used on Zero Sized Types components!"
            );
        }
        let handle = get_comp_handle!(
            self.component_handles,
            T,
            "Tried to get_components_map_mut for unregistered components {:?}!"
        );
        let storage = self.component_manager.get_components_storage(*handle);

        Components_Map_Safe::new(storage)
    }

    /// Like `get_components_map`, but allows mutating components. This is more unsafe to use
    /// as it allows the coexistence of maps that allow mutable access to components.
    /// Note that while a `Components_Map_Unsafe` exists, all other Maps simultaneously
    /// existing must be created via this method, even if they don't need mutable access.
    ///
    /// # Safety
    /// Only at most one map per type must exist at the same time: it is UB to call
    /// `get_components_map_mut::<T>()` multiple times with the same `T`.
    pub fn get_components_map_unsafe<T: 'static + Copy>(&mut self) -> Components_Map_Unsafe<T> {
        let comp_size = std::mem::size_of::<T>();
        if comp_size == 0 {
            panic!(
                "[ ERROR ] get_components_map_mut cannot be used on Zero Sized Types components!"
            );
        }
        let handle = get_comp_handle!(
            self.component_handles,
            T,
            "Tried to get_components_map_mut for unregistered components {:?}!"
        );
        let storage = self.component_manager.get_components_storage_mut(*handle);

        Components_Map_Unsafe::new(storage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Debug, Default)]
    struct C_Test {
        pub foo: i32,
    }

    #[derive(Copy, Clone, Debug, Default)]
    struct C_Test2 {
        foo: i32,
    }

    #[derive(Copy, Clone, Debug, Default)]
    struct C_Test3 {
        foo: i32,
    }

    #[derive(Copy, Clone, Debug, PartialEq)]
    struct C_Test_NonZeroDefault {
        pub foo: i32,
    }

    impl Default for C_Test_NonZeroDefault {
        fn default() -> Self {
            C_Test_NonZeroDefault { foo: 42 }
        }
    }

    #[derive(Copy, Clone, Default)]
    struct C_ZST {}

    #[test]
    #[should_panic]
    fn register_same_component_twice() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        em.register_component::<C_Test>();
    }

    #[test]
    fn add_component_default_value() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test_NonZeroDefault>();

        let e = em.new_entity();
        let test = em.add_component::<C_Test_NonZeroDefault>(e);
        assert_eq!(*test, C_Test_NonZeroDefault::default());
    }

    #[test]
    fn add_component_modify() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();

        let e = em.new_entity();
        let foo = em.add_component::<C_Test>(e);
        assert_eq!(foo.foo, 0);

        foo.foo = 42;
        let foo = em.get_component::<C_Test>(e).unwrap();
        assert_eq!(foo.foo, 42);
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
    fn get_component_zero_sized() {
        let mut em = Ecs_World::new();
        em.register_component::<C_ZST>();

        let e = em.new_entity();
        assert!(em.get_component::<C_ZST>(e).is_none());

        let e2 = em.new_entity();

        em.add_component::<C_ZST>(e);
        em.add_component::<C_ZST>(e2);
        assert!(em.get_component::<C_ZST>(e).is_some());
        assert!(em.get_component::<C_ZST>(e2).is_some());

        em.remove_component::<C_ZST>(e);
        assert!(em.get_component::<C_ZST>(e).is_none());
        assert!(em.get_component::<C_ZST>(e2).is_some());

        em.remove_component::<C_ZST>(e2);
        assert!(em.get_component::<C_ZST>(e2).is_none());
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
    fn get_components_size() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        for _i in 0..10 {
            let e = em.new_entity();
            em.add_component::<C_Test>(e);
        }
        assert_eq!(em.get_components::<C_Test>().len(), 10);
    }

    #[test]
    fn get_components_size_zst() {
        let mut em = Ecs_World::new();
        em.register_component::<C_ZST>();
        for _i in 0..10 {
            let e = em.new_entity();
            em.add_component::<C_ZST>(e);
        }
        // get_components on a ZST component should always be zero-length
        assert_eq!(em.get_components::<C_ZST>().len(), 0);
    }

    #[test]
    fn get_components_size_empty() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        assert_eq!(em.get_components::<C_Test>().len(), 0);
    }

    #[test]
    #[should_panic]
    fn get_unregistered_components() {
        let em = Ecs_World::new();
        em.get_components::<C_Test>();
    }

    #[test]
    fn get_components_mut_mutability() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        for _i in 0..10 {
            let e = em.new_entity();
            em.add_component::<C_Test>(e);
        }
        for (i, test) in em.get_components_mut::<C_Test>().iter_mut().enumerate() {
            test.foo = i as i32;
        }
        for (i, test) in em.get_components::<C_Test>().iter().enumerate() {
            assert_eq!(test.foo, i as i32);
        }
    }

    #[test]
    fn get_components_mut_size() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        for _i in 0..10 {
            let e = em.new_entity();
            em.add_component::<C_Test>(e);
        }
        assert_eq!(em.get_components_mut::<C_Test>().len(), 10);
    }

    #[test]
    fn get_components_mut_size_zst() {
        let mut em = Ecs_World::new();
        em.register_component::<C_ZST>();
        for _i in 0..10 {
            let e = em.new_entity();
            em.add_component::<C_ZST>(e);
        }
        // get_components on a ZST component should always be zero-length
        assert_eq!(em.get_components_mut::<C_ZST>().len(), 0);
    }

    #[test]
    fn get_components_mut_size_empty() {
        let mut em = Ecs_World::new();
        em.register_component::<C_Test>();
        assert_eq!(em.get_components_mut::<C_Test>().len(), 0);
    }

    #[test]
    #[should_panic]
    fn get_unregistered_components_mut() {
        let mut em = Ecs_World::new();
        em.get_components_mut::<C_Test>();
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

    #[test]
    fn components_map() {
        let mut em = Ecs_World::new();
        let mut entities: Vec<Entity> = vec![];
        em.register_component::<C_Test>();

        for _ in 0..10 {
            let e = em.new_entity();
            em.add_component::<C_Test>(e);
            entities.push(e);
        }

        let map = em.get_components_map::<C_Test>();

        for e in entities {
            assert!(map.get_component(e).is_some());
            assert_eq!(map.get_component(e).unwrap().foo, 0);
        }
    }

    #[test]
    fn components_map_unsafe() {
        let mut em = Ecs_World::new();
        let mut entities: Vec<Entity> = vec![];
        em.register_component::<C_Test>();

        for _ in 0..10 {
            let e = em.new_entity();
            em.add_component::<C_Test>(e);
            entities.push(e);
        }

        let mut map = em.get_components_map_unsafe::<C_Test>();

        for (i, e) in entities.iter().enumerate() {
            let e = *e;
            unsafe {
                assert!(map.get_component(e).is_some());
                assert!(map.get_component_mut(e).is_some());
                assert_eq!(map.get_component(e).unwrap().foo, 0);

                map.get_component_mut(e).unwrap().foo = i as i32;
            }
        }

        for (i, e) in entities.iter().enumerate() {
            assert_eq!(unsafe { map.get_component(*e) }.unwrap().foo, i as i32);
        }
    }

    #[test]
    #[should_panic(
        expected = "[ ERROR ] get_components_map_mut cannot be used on Zero Sized Types components!"
    )]
    fn components_map_zst() {
        let mut em = Ecs_World::new();
        em.register_component::<C_ZST>();
        let _map = em.get_components_map::<C_ZST>();
    }
}
