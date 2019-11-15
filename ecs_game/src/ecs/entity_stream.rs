use super::entity_manager_new::{Ecs_World, Entity};
use ecs_engine::core::common::bitset::Bit_Set;
use std::any::type_name;

pub struct Entity_Stream<'a> {
    world: &'a Ecs_World,
    required_components: Bit_Set,
    cur_idx: usize,
}

impl<'a> Entity_Stream<'a> {
    pub fn new(world: &'a Ecs_World) -> Self {
        Entity_Stream {
            world,
            required_components: Bit_Set::default(),
            cur_idx: 0,
        }
    }

    /// Adds component 'T' to the required components
    pub fn require<T: 'static + Copy>(mut self) -> Self {
        let handle = self
            .world
            .component_handles
            .get(&std::any::TypeId::of::<T>())
            .unwrap_or_else(|| panic!("Requiring inexisting component {}!", type_name::<T>()));
        self.required_components.set(*handle as usize, true);
        self
    }
}

impl Iterator for Entity_Stream<'_> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.cur_idx;
        let req_comps = &self.required_components;
        let entity_comp_set = &self.world.component_manager.entity_comp_set;
        for (i, comp_set) in entity_comp_set.iter().enumerate().skip(self.cur_idx) {
            if &(comp_set & req_comps) != req_comps {
                continue;
            }

            self.cur_idx = i + 1;
            return Some(Entity {
                index: i,
                gen: self.world.entity_manager.cur_gen(i),
            });
        }

        self.cur_idx = i;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone)]
    struct C_Test {
        pub foo: u32,
    }

    #[derive(Copy, Clone)]
    struct C_Test2 {}

    #[test]
    fn entity_stream_required_components() {
        let mut world = Ecs_World::new();
        world.register_component::<C_Test>();
        world.register_component::<C_Test2>();

        let e = world.new_entity();
        world.add_component::<C_Test>(e);
        world.add_component::<C_Test2>(e);

        let _e2 = world.new_entity();
        let e3 = world.new_entity();
        world.add_component::<C_Test>(e3);
        let e4 = world.new_entity();
        world.add_component::<C_Test>(e4);
        world.add_component::<C_Test2>(e4);
        let e5 = world.new_entity();
        world.add_component::<C_Test2>(e5);

        let mut stream = Entity_Stream::new(&world)
            .require::<C_Test>()
            .require::<C_Test2>();

        assert_eq!(stream.next(), Some(e));
        assert_eq!(stream.next(), Some(e4));
        assert_eq!(stream.next(), None);
    }
}
