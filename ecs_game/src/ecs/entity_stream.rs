use super::entity_manager_new::{Ecs_World, Entity};
use ecs_engine::core::common::bitset::Bit_Set;

pub struct Entity_Stream<'a> {
    world: &'a Ecs_World,
    required_components: Bit_Set,
    cur_idx: usize,
}

impl<'a> Entity_Stream<'a> {
    pub fn new(world: &'a Ecs_World, required_components: Bit_Set) -> Self {
        Entity_Stream {
            world,
            required_components,
            cur_idx: 0,
        }
    }
}

impl Iterator for Entity_Stream<'_> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.cur_idx;
        let req_comps = &self.required_components;
        let entity_comp_set = &self.world.component_manager.entity_comp_set;
        for i in self.cur_idx..entity_comp_set.len() {
            let comp_set = &entity_comp_set[i];

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
        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typename::TypeName;

    #[derive(Copy, Clone, TypeName)]
    struct C_Test {
        pub foo: u32,
    }

    #[test]
    fn entity_stream_required_components() {
        let mut world = Ecs_World::new();
        world.register_component::<C_Test>();

        let e = world.new_entity();
        world.add_component::<C_Test>(e);

        let e2 = world.new_entity();
        let e3 = world.new_entity();
        let e4 = world.new_entity();
        world.add_component::<C_Test>(e4);

        let mut stream = {
            let c_test_handle = world
                .component_handles
                .get(&std::any::TypeId::of::<C_Test>())
                .unwrap();
            let mut req_comps = Bit_Set::default();
            req_comps.set(*c_test_handle as usize, true);
            Entity_Stream::new(&world, req_comps)
        };

        assert_eq!(stream.next(), Some(e));
        assert_eq!(stream.next(), Some(e4));
        assert_eq!(stream.next(), None);
    }
}
