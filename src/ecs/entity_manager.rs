use super::components::Component;
use crate::alloc::generational_allocator::{Generational_Allocator, Generational_Index};

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
}

pub type Entity = Generational_Index;
type VecOpt<T> = Vec<Option<RefCell<T>>>;

impl Entity_Manager {
    pub fn new() -> Entity_Manager {
        Entity_Manager {
            allocator: Generational_Allocator::new(4),
            components: AnyMap::new(),
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
            .expect(&format!(
                "Tried to get_components of unregistered type {}!",
                C::type_name()
            ))
            .iter()
            .filter_map(|c| Some(c.as_ref()?.borrow()))
            .collect()
    }

    pub fn get_components_mut<C>(&mut self) -> Vec<RefMut<'_, C>>
    where
        C: Component + 'static,
    {
        self.get_comp_storage_mut::<C>()
            .expect(&format!(
                "Tried to get_components of unregistered type {}!",
                C::type_name()
            ))
            .iter_mut()
            .filter_map(|c| Some(c.as_ref()?.borrow_mut()))
            .collect()
    }

    pub fn new_entity(&mut self) -> Entity {
        self.allocator.allocate()
    }

    pub fn is_valid_entity(&self, e: Entity) -> bool {
        self.allocator.is_valid(e)
    }

    pub fn destroy_entity(&mut self, e: Entity) {
        self.allocator.deallocate(e);
    }

    pub fn register_component<C>(&mut self)
    where
        C: Component + 'static,
    {
        if let Some(_) = self.get_comp_storage::<C>() {
            panic!(
                "Tried to register the same component {} twice!",
                C::type_name()
            );
        }
        let v: VecOpt<C> = Vec::new();
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

        let alloc_size = self.allocator.size();
        match self.get_comp_storage_mut::<C>() {
            Some(vec) => {
                vec.resize(alloc_size, None);
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

        match self.get_comp_storage_mut::<C>() {
            Some(vec) => {
                vec[e.index] = None;
            } // We don't assert if component is already None.
            None => panic!(
                "Tried to remove unregistered component {} to entity!",
                C::type_name()
            ),
        }
    }

    pub fn get_component<'a, C>(&'a self, e: Entity) -> Option<Ref<'a, C>>
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

    // @Refactoring: this code is almost exactly the same as `get_component`. Can we do something about it?
    pub fn get_component_mut<'a, C>(&'a mut self, e: Entity) -> Option<RefMut<'a, C>>
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

    pub fn get_component_tuple<'a, C1, C2>(
        &'a self,
    ) -> impl Iterator<Item = (Ref<'a, C1>, Ref<'a, C2>)>
    where
        C1: Component + 'static,
        C2: Component + 'static,
    {
        let comps1 = self
            .get_comp_storage::<C1>()
            .expect(format!("Tried to get unregistered component {}!", C1::type_name()).as_str());
        let comps2 = self
            .get_comp_storage::<C2>()
            .expect(format!("Tried to get unregistered component {}!", C2::type_name()).as_str());

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
            .expect(format!("Tried to get unregistered component {}!", C1::type_name()).as_str());
        let comps2 = self
            .get_comp_storage::<C2>()
            .expect(format!("Tried to get unregistered component {}!", C2::type_name()).as_str());

        comps1.iter().zip(comps2.iter()).filter_map(|(c1, c2)| {
            let c1 = c1.as_ref()?;
            let c2 = c2.as_ref()?;
            Some((c1, c2))
        })
    }
}

#[cfg(test)]
mod tests_entity_manager {
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
    fn test_register_same_component_twice() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        em.register_component::<C_Test>();
    }

    #[test]
    fn test_get_component() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();

        let e = em.new_entity();
        assert!(em.get_component::<C_Test>(e).is_none());

        em.add_component::<C_Test>(e);
        assert!(em.get_component::<C_Test>(e).is_some());
    }

    #[test]
    fn test_get_component_mut() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();

        let e = em.new_entity();
        assert!(em.get_component_mut::<C_Test>(e).is_none());

        em.add_component::<C_Test>(e);
        assert!(em.get_component_mut::<C_Test>(e).is_some());
    }

    #[test]
    fn test_mutate_component() {
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
    fn test_add_component_inexisting_entity() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        em.add_component::<C_Test>(Entity { index: 0, gen: 1 });
    }

    #[test]
    #[should_panic]
    fn test_get_component_inexisting_entity() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        em.get_component::<C_Test>(Entity { index: 0, gen: 1 });
    }

    #[test]
    #[should_panic]
    fn test_get_component_mut_inexisting_entity() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        em.get_component_mut::<C_Test>(Entity { index: 0, gen: 1 });
    }

    #[test]
    fn test_destroy_entity() {
        let mut em = Entity_Manager::new();
        let e = em.new_entity();
        em.destroy_entity(e);
    }

    #[test]
    #[should_panic]
    fn test_double_free_entity() {
        let mut em = Entity_Manager::new();
        let e = em.new_entity();
        em.destroy_entity(e);
        em.destroy_entity(e);
    }

    #[test]
    #[should_panic]
    fn test_destroy_inexisting_entity() {
        let mut em = Entity_Manager::new();
        em.destroy_entity(Entity { index: 0, gen: 1 });
    }

    #[test]
    #[should_panic]
    fn test_add_component_destroyed_entity() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.destroy_entity(e);
        em.add_component::<C_Test>(e);
    }

    #[test]
    #[should_panic]
    fn test_get_component_destroyed_entity() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.destroy_entity(e);
        em.get_component::<C_Test>(e);
    }

    #[test]
    #[should_panic]
    fn test_get_component_destroyed_and_recreated_entity() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.destroy_entity(e);
        em.new_entity();
        em.get_component::<C_Test>(e);
    }

    #[test]
    fn test_get_component_destroyed_and_recreated_entity_good() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();

        let e1 = em.new_entity();
        em.add_component::<C_Test>(e1);
        em.destroy_entity(e1);

        let e2 = em.new_entity();
        em.get_component::<C_Test>(e2);
    }

    #[test]
    fn test_remove_component() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.remove_component::<C_Test>(e);
    }

    #[test]
    fn test_double_remove_component() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.remove_component::<C_Test>(e);
        em.remove_component::<C_Test>(e);
    }

    #[test]
    fn test_get_removed_component() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.remove_component::<C_Test>(e);
        assert!(em.get_component::<C_Test>(e).is_none());
    }

    #[test]
    fn test_remove_and_readd_component() {
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
    fn test_remove_component_destroyed_and_recreated_entity() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        let e = em.new_entity();
        em.add_component::<C_Test>(e);
        em.destroy_entity(e);
        em.new_entity();
        em.remove_component::<C_Test>(e);
    }

    #[test]
    fn test_get_components_size() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        for _i in 0..10 {
            let e = em.new_entity();
            em.add_component::<C_Test>(e);
        }
        assert_eq!(em.get_components::<C_Test>().len(), 10);
    }

    #[test]
    fn test_get_components_size_empty() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        assert_eq!(em.get_components::<C_Test>().len(), 0);
    }

    #[test]
    #[should_panic]
    fn test_get_unregistered_components() {
        let em = Entity_Manager::new();
        em.get_components::<C_Test>();
    }

    #[test]
    fn test_get_components_mut_size() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        for _i in 0..10 {
            let e = em.new_entity();
            em.add_component::<C_Test>(e);
        }
        assert_eq!(em.get_components_mut::<C_Test>().len(), 10);
    }

    #[test]
    fn test_get_components_mut_size_empty() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Test>();
        assert_eq!(em.get_components_mut::<C_Test>().len(), 0);
    }

    #[test]
    #[should_panic]
    fn test_get_unregistered_components_mut() {
        let mut em = Entity_Manager::new();
        em.get_components_mut::<C_Test>();
    }

    #[test]
    fn test_has_get_consistency() {
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
    fn test_has_get_consistency_2() {
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
    fn test_get_component_tuple() {
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
    fn test_get_component_tuple_empty() {
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
    fn test_get_component_tuple_mut() {
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
    fn test_get_component_tuple_mut_empty() {
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
}
