// This file is meant to be include!-ed in ecs_world.rs

#[cfg(all(test, not(miri)))]
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

        let mut map = unsafe { em.get_components_map_unsafe::<C_Test>() };

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
        expected = "[ FATAL ] get_components_map_mut cannot be used on Zero Sized Types components!"
    )]
    fn components_map_zst() {
        let mut em = Ecs_World::new();
        em.register_component::<C_ZST>();
        let _map = em.get_components_map::<C_ZST>();
    }
}
