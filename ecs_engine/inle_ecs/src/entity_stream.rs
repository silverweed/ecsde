use super::ecs_world::{Ecs_World, Entity};
use inle_alloc::temp::Exclusive_Temp_Array;
use inle_common::bitset::Bit_Set;
use std::borrow::Borrow;

pub struct Entity_Stream {
    required_components: Bit_Set,
    excluded_components: Bit_Set,
    cur_idx: usize,
}

impl Entity_Stream {
    pub fn next(&mut self, world: &Ecs_World) -> Option<Entity> {
        let req_comps = &self.required_components;
        let exc_comps = &self.excluded_components;
        let entities = world.entities();
        for (i, &entity) in entities.iter().enumerate().skip(self.cur_idx) {
            let comp_set = world.get_entity_comp_set(entity);
            let comp_set = comp_set.borrow();

            {
                trace!("entity_stream::bitset_test");

                if (comp_set & req_comps) != *req_comps {
                    continue;
                }

                if (comp_set & exc_comps) != Bit_Set::default() {
                    continue;
                }
            }

            self.cur_idx = i + 1;
            return Some(entity);
        }

        self.cur_idx = entities.len();
        None
    }

    /// Fills `vec` with all entities retrieved calling `next` on self.
    /// Does not clear the Vec!
    pub fn collect<T: Pushable<Entity>>(&mut self, world: &Ecs_World, vec: &mut T) {
        while let Some(entity) = self.next(world) {
            vec.push(entity);
        }
    }
}

pub struct Entity_Stream_Builder<'a> {
    world: &'a Ecs_World,
    required_components: Bit_Set,
    excluded_components: Bit_Set,
}

impl Entity_Stream_Builder<'_> {
    /// Adds component 'T' to the required components
    pub fn require<T: 'static + Copy>(mut self) -> Self {
        let handle = self.world.component_manager.get_handle::<T>();
        self.required_components.set(handle as usize, true);
        self
    }

    /// Adds component 'T' to the excluded components
    pub fn exclude<T: 'static + Copy>(mut self) -> Self {
        let handle = self.world.component_manager.get_handle::<T>();
        self.excluded_components.set(handle as usize, true);
        self
    }

    pub fn build(self) -> Entity_Stream {
        Entity_Stream {
            required_components: self.required_components,
            excluded_components: self.excluded_components,
            cur_idx: 0,
        }
    }
}

pub fn new_entity_stream(world: &Ecs_World) -> Entity_Stream_Builder {
    Entity_Stream_Builder {
        world,
        required_components: Bit_Set::default(),
        excluded_components: Bit_Set::default(),
    }
}

pub trait Pushable<T> {
    fn push(&mut self, elem: T);
}

impl<T> Pushable<T> for Vec<T> {
    fn push(&mut self, elem: T) {
        self.push(elem);
    }
}

impl<T: Copy> Pushable<T> for Exclusive_Temp_Array<'_, T> {
    fn push(&mut self, elem: T) {
        self.push(elem);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Default)]
    struct C_Test {
        pub foo: u32,
    }

    #[derive(Copy, Clone, Default)]
    struct C_Test2 {}

    #[test]
    fn entity_stream_required_components() {
        let mut world = Ecs_World::new();
        world.register_component::<C_Test>();
        world.register_component::<C_Test2>();

        let e = world.new_entity();
        world.add_component(e, C_Test::default());
        world.add_component(e, C_Test2::default());

        let _e2 = world.new_entity();
        let e3 = world.new_entity();
        world.add_component(e3, C_Test::default());
        let e4 = world.new_entity();
        world.add_component(e4, C_Test::default());
        world.add_component(e4, C_Test2::default());
        let e5 = world.new_entity();
        world.add_component(e5, C_Test2::default());

        let mut stream = new_entity_stream(&world)
            .require::<C_Test>()
            .require::<C_Test2>()
            .build();

        assert_eq!(stream.next(&world), Some(e));
        assert_eq!(stream.next(&world), Some(e4));
        assert_eq!(stream.next(&world), None);
    }

    #[test]
    fn entity_stream_excluded_components() {
        let mut world = Ecs_World::new();
        world.register_component::<C_Test>();
        world.register_component::<C_Test2>();

        let e = world.new_entity();
        world.add_component(e, C_Test::default());
        world.add_component(e, C_Test2::default());

        let e2 = world.new_entity();
        let e3 = world.new_entity();
        world.add_component(e3, C_Test::default());
        let e4 = world.new_entity();
        world.add_component(e4, C_Test::default());
        world.add_component(e4, C_Test2::default());
        let e5 = world.new_entity();
        world.add_component(e5, C_Test2::default());

        let mut stream = new_entity_stream(&world).exclude::<C_Test2>().build();

        assert_eq!(stream.next(&world), Some(e2));
        assert_eq!(stream.next(&world), Some(e3));
        assert_eq!(stream.next(&world), None);
    }

    #[test]
    fn entity_stream_required_excluded_components() {
        let mut world = Ecs_World::new();
        world.register_component::<C_Test>();
        world.register_component::<C_Test2>();

        let e = world.new_entity();
        world.add_component(e, C_Test::default());
        world.add_component(e, C_Test2::default());

        let _e2 = world.new_entity();
        let e3 = world.new_entity();
        world.add_component(e3, C_Test::default());
        let e4 = world.new_entity();
        world.add_component(e4, C_Test::default());
        world.add_component(e4, C_Test2::default());
        let e5 = world.new_entity();
        world.add_component(e5, C_Test2::default());

        let mut stream = new_entity_stream(&world)
            .require::<C_Test>()
            .exclude::<C_Test2>()
            .build();

        assert_eq!(stream.next(&world), Some(e3));
        assert_eq!(stream.next(&world), None);
    }
}
