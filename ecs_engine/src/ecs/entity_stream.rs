use super::ecs_world::{Ecs_World, Entity};
use crate::common::bitset::Bit_Set;
use std::any::type_name;
use crate::alloc::temp::{Temp_Array, Exclusive_Temp_Array};
use std::convert::TryFrom;

pub struct Entity_Stream {
    required_components: Bit_Set,
    excluded_components: Bit_Set,
    cur_idx: usize,
}

impl Entity_Stream {
    pub fn next(&mut self, world: &Ecs_World) -> Option<Entity> {
        let i = self.cur_idx;
        let req_comps = &self.required_components;
        let exc_comps = &self.excluded_components;
        let entity_comp_set = &world.component_manager.entity_comp_set;
        for (i, comp_set) in entity_comp_set.iter().enumerate().skip(self.cur_idx) {
            if (comp_set & req_comps) != *req_comps {
                continue;
            }

            if (comp_set & exc_comps) != Bit_Set::default() {
                continue;
            }

            self.cur_idx = i + 1;
            let index = u32::try_from(i).unwrap_or_else(|_| {
                fatal!("Entity_Stream::next(): index overflowed u32! ({})", i);
            });
            return Some(Entity {
                index,
                gen: world.entity_manager.cur_gen(index),
            });
        }

        self.cur_idx = i;
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
        let handle = self
            .world
            .component_handles
            .get(&std::any::TypeId::of::<T>())
            .unwrap_or_else(|| fatal!("Requiring inexisting component {}!", type_name::<T>()));
        self.required_components.set(*handle as usize, true);
        self
    }

    /// Adds component 'T' to the excluded components
    pub fn exclude<T: 'static + Copy>(mut self) -> Self {
        let handle = self
            .world
            .component_handles
            .get(&std::any::TypeId::of::<T>())
            .unwrap_or_else(|| fatal!("Requiring inexisting component {}!", type_name::<T>()));
        self.excluded_components.set(*handle as usize, true);
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

impl<T: Copy> Pushable<T> for Temp_Array<'_, T> {
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
        world.add_component::<C_Test>(e);
        world.add_component::<C_Test2>(e);

        let e2 = world.new_entity();
        let e3 = world.new_entity();
        world.add_component::<C_Test>(e3);
        let e4 = world.new_entity();
        world.add_component::<C_Test>(e4);
        world.add_component::<C_Test2>(e4);
        let e5 = world.new_entity();
        world.add_component::<C_Test2>(e5);

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

        let mut stream = new_entity_stream(&world)
            .require::<C_Test>()
            .exclude::<C_Test2>()
            .build();

        assert_eq!(stream.next(&world), Some(e3));
        assert_eq!(stream.next(&world), None);
    }
}
