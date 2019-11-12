use super::components::Component;
use ecs_engine::alloc::generational_allocator::{Generational_Allocator, Generational_Index};

use std::cell::{Ref, RefCell, RefMut};
use std::iter::Iterator;
use std::option::Option;
use std::vec::Vec;

use anymap::AnyMap;

/// An Entity_Manager provides the public interface to allocate/deallocate Entities
/// along with their Components' storage. It allows to add/remove/query Components
/// to/from their associated Entity.
pub struct Entity_Manager {
    allocator: Generational_Allocator,
    // map { CompType => Vec<CompType> }
    components: AnyMap,
    #[cfg(debug_assertions)]
    debug_info: Entity_Manager_Debug_Info,
}

#[cfg(debug_assertions)]
pub struct Entity_Manager_Debug_Info {
    /// Number of bytes used by "live" components
    pub components_used_bytes: usize,
    pub components_total_bytes: usize,
    pub n_component_types_registered: u16,
    pub n_components_currently_instantiated: usize,
    pub max_n_components_ever_instantiated: usize,
    pub n_entities_currently_instantiated: usize,
    pub max_n_entities_ever_instantiated: usize,
}

#[cfg(debug_assertions)]
impl std::fmt::Display for Entity_Manager_Debug_Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "
Component used bytes:                {} B ({:.2}% occupancy)
Component total bytes:               {} B
# Component types registered:        {}
# Components currently instantiated: {}
Max # Components ever instantiated:  {}
# Entities currently instantiated:   {}
Max # Entities ever instantiated:    {}",
            self.components_used_bytes,
            100.0 * (self.components_used_bytes as f32) / (self.components_total_bytes as f32),
            self.components_total_bytes,
            self.n_component_types_registered,
            self.n_components_currently_instantiated,
            self.max_n_components_ever_instantiated,
            self.n_entities_currently_instantiated,
            self.max_n_entities_ever_instantiated
        )
    }
}

pub type Entity = Generational_Index;
type VecOpt<T> = Vec<Option<RefCell<T>>>;

impl Entity_Manager {
    const INITIAL_SIZE: usize = 64;

    #[cfg(not(debug_assertions))]
    pub fn new() -> Entity_Manager {
        Entity_Manager {
            allocator: Generational_Allocator::new(Self::INITIAL_SIZE),
            components: AnyMap::new(),
        }
    }

    #[cfg(debug_assertions)]
    pub fn new() -> Entity_Manager {
        Entity_Manager {
            allocator: Generational_Allocator::new(Self::INITIAL_SIZE),
            components: AnyMap::new(),
            debug_info: Entity_Manager_Debug_Info {
                components_used_bytes: 0,
                components_total_bytes: 0,
                n_component_types_registered: 0,
                n_components_currently_instantiated: 0,
                max_n_components_ever_instantiated: 0,
                n_entities_currently_instantiated: 0,
                max_n_entities_ever_instantiated: 0,
            },
        }
    }

    fn get_comp_storage<C>(&self) -> Option<&VecOpt<C>>
    where
        C: Component + 'static,
    {
        self.components.get::<VecOpt<C>>()
    }

    fn get_comp_storage_mut<C>(&mut self) -> Option<&mut VecOpt<C>>
    where
        C: Component + 'static,
    {
        self.components.get_mut::<VecOpt<C>>()
    }

    pub fn get_components<C>(&self) -> Vec<Ref<'_, C>>
    where
        C: Component + 'static,
    {
        self.get_comp_storage::<C>()
            .unwrap_or_else(|| {
                panic!(
                    "Tried to get_components of unregistered type {}!",
                    C::type_name()
                )
            })
            .iter()
            .filter_map(|c| Some(c.as_ref()?.borrow()))
            .collect()
    }

    pub fn get_components_mut<C>(&mut self) -> Vec<RefMut<'_, C>>
    where
        C: Component + 'static,
    {
        self.get_comp_storage_mut::<C>()
            .unwrap_or_else(|| {
                panic!(
                    "Tried to get_components of unregistered type {}!",
                    C::type_name()
                )
            })
            .iter_mut()
            .filter_map(|c| Some(c.as_ref()?.borrow_mut()))
            .collect()
    }

    pub fn new_entity(&mut self) -> Entity {
        #[cfg(debug_assertions)]
        {
            self.debug_info.n_entities_currently_instantiated += 1;
            self.debug_info.max_n_entities_ever_instantiated = self
                .debug_info
                .max_n_entities_ever_instantiated
                .max(self.debug_info.n_entities_currently_instantiated);
        }
        self.allocator.allocate()
    }

    pub fn is_valid_entity(&self, e: Entity) -> bool {
        self.allocator.is_valid(e)
    }

    pub fn destroy_entity(&mut self, e: Entity) {
        #[cfg(debug_assertions)]
        {
            self.debug_info.n_entities_currently_instantiated -= 1;
        }
        self.allocator.deallocate(e);
    }

    pub fn register_component<C>(&mut self)
    where
        C: Component + 'static,
    {
        if self.get_comp_storage::<C>().is_some() {
            panic!(
                "Tried to register the same component {} twice!",
                C::type_name()
            );
        }
        let v: VecOpt<C> = Vec::new();
        #[cfg(debug_assertions)]
        {
            self.debug_info.n_component_types_registered += 1;
        }
        self.components.insert(v);
    }

    /// Adds a component of type C to `e` and returns a mutable reference to it.
    pub fn add_component<C>(&mut self, e: Entity) -> RefMut<'_, C>
    where
        C: Component + 'static,
    {
        if !self.is_valid_entity(e) {
            panic!(
                "Tried to add component {} to invalid entity {:?}",
                C::type_name(),
                e
            );
        }

        // @Hack: this is to work around the borrow checker
        #[cfg(debug_assertions)]
        let mut dbg = &mut self.debug_info as *mut Entity_Manager_Debug_Info;

        let alloc_size = self.allocator.capacity();
        match self.get_comp_storage_mut::<C>() {
            Some(vec) => {
                #[cfg(debug_assertions)]
                let old_size = std::mem::size_of::<C>() * vec.len();

                vec.resize(alloc_size, None);

                #[cfg(debug_assertions)]
                let delta_size = std::mem::size_of::<C>() * vec.len() - old_size;

                #[cfg(debug_assertions)]
                unsafe {
                    (*dbg).n_components_currently_instantiated += 1;
                    (*dbg).max_n_components_ever_instantiated = (*dbg)
                        .max_n_components_ever_instantiated
                        .max((*dbg).n_components_currently_instantiated);
                    (*dbg).components_used_bytes += std::mem::size_of::<C>();
                    (*dbg).components_total_bytes += delta_size;
                }
                vec[e.index] = Some(RefCell::new(C::default()));
                vec[e.index].as_ref().unwrap().borrow_mut()
            }
            None => panic!(
                "Tried to add unregistered component {} to entity!",
                C::type_name()
            ),
        }
    }

    pub fn remove_component<C>(&mut self, e: Entity)
    where
        C: Component + 'static,
    {
        if !self.is_valid_entity(e) {
            panic!(
                "Tried to remove component {} from invalid entity {:?}",
                C::type_name(),
                e
            );
        }

        // @Hack: this is to work around the borrow checker
        #[cfg(debug_assertions)]
        let mut dbg = &mut self.debug_info as *mut Entity_Manager_Debug_Info;

        match self.get_comp_storage_mut::<C>() {
            Some(vec) => {
                #[cfg(debug_assertions)]
                unsafe {
                    if vec[e.index].is_some() {
                        (*dbg).n_components_currently_instantiated -= 1;
                        (*dbg).components_used_bytes -= std::mem::size_of::<C>();
                    }
                }
                vec[e.index] = None;
            } // We don't assert if component is already None.
            None => panic!(
                "Tried to remove unregistered component {} to entity!",
                C::type_name()
            ),
        }
    }

    pub fn get_component<C>(&self, e: Entity) -> Option<Ref<'_, C>>
    where
        C: Component + 'static,
    {
        if !self.is_valid_entity(e) {
            panic!("Tried to get component of invalid entity {:?}", e);
        }

        match self.get_comp_storage::<C>() {
            Some(vec) => {
                // Note: we may not have added any component yet, so the components Vec is of len 0
                if e.index < vec.len() {
                    if let Some(opt) = vec[e.index].as_ref() {
                        Some(opt.borrow())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            None => panic!("Tried to get unregistered component {}!", C::type_name()),
        }
    }

    pub fn get_component_mut<C>(&mut self, e: Entity) -> Option<RefMut<'_, C>>
    where
        C: Component + 'static,
    {
        if !self.is_valid_entity(e) {
            panic!("Tried to get component of invalid entity {:?}", e);
        }

        match self.get_comp_storage_mut::<C>() {
            Some(vec) => {
                if e.index < vec.len() {
                    if let Some(opt) = vec[e.index].as_mut() {
                        Some(opt.borrow_mut())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            None => panic!("Tried to get unregistered component {}!", C::type_name()),
        }
    }

    pub fn has_component<C>(&self, e: Entity) -> bool
    where
        C: Component + 'static,
    {
        self.get_component::<C>(e).is_some()
    }

    pub fn get_component_tuple<C1, C2>(&self) -> impl Iterator<Item = (Ref<'_, C1>, Ref<'_, C2>)>
    where
        C1: Component + 'static,
        C2: Component + 'static,
    {
        let comps1 = self
            .get_comp_storage::<C1>()
            .unwrap_or_else(|| panic!("Tried to get unregistered component {}!", C1::type_name()));
        let comps2 = self
            .get_comp_storage::<C2>()
            .unwrap_or_else(|| panic!("Tried to get unregistered component {}!", C2::type_name()));

        comps1.iter().zip(comps2.iter()).filter_map(|(c1, c2)| {
            let c1 = c1.as_ref()?.borrow();
            let c2 = c2.as_ref()?.borrow();
            Some((c1, c2))
        })
    }

    pub fn get_component_tuple_mut<C1, C2>(
        &self,
    ) -> impl Iterator<Item = (&RefCell<C1>, &RefCell<C2>)> + '_
    where
        C1: Component + 'static,
        C2: Component + 'static,
    {
        let comps1 = self
            .get_comp_storage::<C1>()
            .unwrap_or_else(|| panic!("Tried to get unregistered component {}!", C1::type_name()));
        let comps2 = self
            .get_comp_storage::<C2>()
            .unwrap_or_else(|| panic!("Tried to get unregistered component {}!", C2::type_name()));

        comps1.iter().zip(comps2.iter()).filter_map(|(c1, c2)| {
            let c1 = c1.as_ref()?;
            let c2 = c2.as_ref()?;
            Some((c1, c2))
        })
    }

    #[cfg(debug_assertions)]
    pub fn print_debug_info(&self) {
        eprintln!("{}", self.debug_info);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typename::TypeName;

    #[derive(Copy, Clone, Debug, Default, TypeName)]
    struct C_Test {
        foo: i32,
    }

    #[derive(Copy, Clone, Debug, Default, TypeName)]
    struct C_Test2 {
        foo: i32,
    }

    #[derive(Copy, Clone, Debug, Default, TypeName)]
    struct C_Test3 {
        foo: i32,
    }

    #[test]
    #[should_panic]
    fn register_same_component_twice() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        em.register_component::<C_Test>();
    }

    #[test]
    fn get_component() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();

        let e = em.new_entity();
        assert!(em.get_component::<C_Test>(e).is_none());

        em.add_component::<C_Test>(e);
        assert!(em.get_component::<C_Test>(e).is_some());
    }

    #[test]
    fn get_component_mut() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();

        let e = em.new_entity();
        assert!(em.get_component_mut::<C_Test>(e).is_none());

        em.add_component::<C_Test>(e);
        assert!(em.get_component_mut::<C_Test>(e).is_some());
    }

    #[test]
    fn mutate_component() {
        let mut em = Entity_Manager::new();
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
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        em.add_component::<C_Test>(Entity { index: 0, gen: 1 });
    }

    #[test]
    #[should_panic]
    fn get_component_inexisting_entity() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        em.get_component::<C_Test>(Entity { index: 0, gen: 1 });
    }

    #[test]
    #[should_panic]
    fn get_component_mut_inexisting_entity() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        em.get_component_mut::<C_Test>(Entity { index: 0, gen: 1 });
    }

    #[test]
    fn destroy_entity() {
        let mut em = Entity_Manager::new();
        let e = em.new_entity();
        em.destroy_entity(e);
    }

    #[test]
    #[should_panic]
    fn double_free_entity() {
        let mut em = Entity_Manager::new();
        let e = em.new_entity();
        em.destroy_entity(e);
        em.destroy_entity(e);
    }

    #[test]
    #[should_panic]
    fn destroy_inexisting_entity() {
        let mut em = Entity_Manager::new();
        em.destroy_entity(Entity { index: 0, gen: 1 });
    }

    #[test]
    #[should_panic]
    fn add_component_destroyed_entity() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.destroy_entity(e);
        em.add_component::<C_Test>(e);
    }

    #[test]
    #[should_panic]
    fn get_component_destroyed_entity() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.destroy_entity(e);
        em.get_component::<C_Test>(e);
    }

    #[test]
    #[should_panic]
    fn get_component_destroyed_and_recreated_entity() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.destroy_entity(e);
        em.new_entity();
        em.get_component::<C_Test>(e);
    }

    #[test]
    fn get_component_destroyed_and_recreated_entity_good() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();

        let e1 = em.new_entity();
        em.add_component::<C_Test>(e1);
        em.destroy_entity(e1);

        let e2 = em.new_entity();
        em.get_component::<C_Test>(e2);
    }

    #[test]
    fn remove_component() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.remove_component::<C_Test>(e);
    }

    #[test]
    fn double_remove_component() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.remove_component::<C_Test>(e);
        em.remove_component::<C_Test>(e);
    }

    #[test]
    fn get_removed_component() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.remove_component::<C_Test>(e);
        assert!(em.get_component::<C_Test>(e).is_none());
    }

    #[test]
    fn remove_and_readd_component() {
        let mut em = Entity_Manager::new();
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
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.destroy_entity(e);
        em.new_entity();
        em.remove_component::<C_Test>(e);
    }

    #[test]
    fn get_components_size() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        for _i in 0..10 {
            let e = em.new_entity();
            em.add_component::<C_Test>(e);
        }
        assert_eq!(em.get_components::<C_Test>().len(), 10);
    }

    #[test]
    fn get_components_size_empty() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        assert_eq!(em.get_components::<C_Test>().len(), 0);
    }

    #[test]
    #[should_panic]
    fn get_unregistered_components() {
        let em = Entity_Manager::new();
        em.get_components::<C_Test>();
    }

    #[test]
    fn get_components_mut_size() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        for _i in 0..10 {
            let e = em.new_entity();
            em.add_component::<C_Test>(e);
        }
        assert_eq!(em.get_components_mut::<C_Test>().len(), 10);
    }

    #[test]
    fn get_components_mut_size_empty() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        assert_eq!(em.get_components_mut::<C_Test>().len(), 0);
    }

    #[test]
    #[should_panic]
    fn get_unregistered_components_mut() {
        let mut em = Entity_Manager::new();
        em.get_components_mut::<C_Test>();
    }

    #[test]
    fn has_get_consistency() {
        let mut em = Entity_Manager::new();
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
    fn has_get_consistency_2() {
        let mut em = Entity_Manager::new();
        let mut entities: Vec<Entity> = vec![];
        em.register_component::<C_Test>();
        for _i in 0..66 {
            let e = em.new_entity();
            entities.push(e);
            em.add_component::<C_Test>(e);
        }

        let filtered: Vec<Entity> = entities
            .iter()
            .filter(|&&e| em.has_component::<C_Test>(e))
            .cloned()
            .collect();
        let all_nonnull_comps = em.get_components::<C_Test>();
        assert_eq!(filtered.len(), all_nonnull_comps.len());
    }

    #[test]
    fn get_component_tuple() {
        let mut em = Entity_Manager::new();

        em.register_component::<C_Test>();
        em.register_component::<C_Test2>();

        let has_both_1 = em.new_entity();
        em.add_component::<C_Test>(has_both_1);
        em.add_component::<C_Test2>(has_both_1);

        let has_first = em.new_entity();
        em.add_component::<C_Test>(has_first);

        em.new_entity();

        let has_both_2 = em.new_entity();
        em.add_component::<C_Test>(has_both_2);
        em.add_component::<C_Test2>(has_both_2);

        let has_second = em.new_entity();
        em.add_component::<C_Test>(has_second);

        let has_both_3 = em.new_entity();
        em.add_component::<C_Test>(has_both_3);
        em.add_component::<C_Test2>(has_both_3);

        em.new_entity();

        let only_both: Vec<(Ref<'_, C_Test>, Ref<'_, C_Test2>)> =
            em.get_component_tuple::<C_Test, C_Test2>().collect();
        assert_eq!(only_both.len(), 3);

        let only_both: Vec<(Ref<'_, C_Test2>, Ref<'_, C_Test>)> =
            em.get_component_tuple::<C_Test2, C_Test>().collect();
        assert_eq!(only_both.len(), 3);
    }

    #[test]
    fn get_component_tuple_empty() {
        let mut em = Entity_Manager::new();

        em.register_component::<C_Test>();
        em.register_component::<C_Test2>();
        em.register_component::<C_Test3>();

        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.add_component::<C_Test2>(e);

        let empty: Vec<(Ref<'_, C_Test>, Ref<'_, C_Test3>)> =
            em.get_component_tuple::<C_Test, C_Test3>().collect();
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn get_component_tuple_mut() {
        let mut em = Entity_Manager::new();

        em.register_component::<C_Test>();
        em.register_component::<C_Test2>();

        let has_both_1 = em.new_entity();
        em.add_component::<C_Test>(has_both_1);
        em.add_component::<C_Test2>(has_both_1);

        let has_first = em.new_entity();
        em.add_component::<C_Test>(has_first);

        em.new_entity();

        let has_both_2 = em.new_entity();
        em.add_component::<C_Test>(has_both_2);
        em.add_component::<C_Test2>(has_both_2);

        let has_second = em.new_entity();
        em.add_component::<C_Test>(has_second);

        let has_both_3 = em.new_entity();
        em.add_component::<C_Test>(has_both_3);
        em.add_component::<C_Test2>(has_both_3);

        em.new_entity();

        let only_both: Vec<(&RefCell<C_Test>, &RefCell<C_Test2>)> =
            em.get_component_tuple_mut::<C_Test, C_Test2>().collect();
        assert_eq!(only_both.len(), 3);

        let only_both: Vec<(&RefCell<C_Test2>, &RefCell<C_Test>)> =
            em.get_component_tuple_mut::<C_Test2, C_Test>().collect();
        assert_eq!(only_both.len(), 3);
    }

    #[test]
    fn get_component_tuple_mut_empty() {
        let mut em = Entity_Manager::new();

        em.register_component::<C_Test>();
        em.register_component::<C_Test2>();
        em.register_component::<C_Test3>();

        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.add_component::<C_Test2>(e);

        let empty: Vec<(&RefCell<C_Test>, &RefCell<C_Test3>)> =
            em.get_component_tuple_mut::<C_Test, C_Test3>().collect();
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn get_component_tuple_mut_mutability() {
        let mut em = Entity_Manager::new();

        em.register_component::<C_Test>();
        em.register_component::<C_Test2>();

        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.add_component::<C_Test2>(e);

        for (test, _) in em.get_component_tuple_mut::<C_Test, C_Test2>() {
            test.borrow_mut().foo = 42;
        }

        let test = em.get_component::<C_Test>(e);
        assert_eq!(test.unwrap().foo, 42);
    }

    #[test]
    fn get_component_tuple_mut_borrow_rules() {
        let mut em = Entity_Manager::new();

        em.register_component::<C_Test>();
        em.register_component::<C_Test2>();

        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        {
            let mut t2 = em.add_component::<C_Test2>(e);
            t2.foo = 42;
        }

        for (test, test2) in em.get_component_tuple_mut::<C_Test, C_Test2>() {
            let test2 = test2.borrow();
            let mut test = test.borrow_mut();
            test.foo = test2.foo;
        }

        let test = em.get_component::<C_Test>(e);
        assert_eq!(test.unwrap().foo, 42);
    }
}
