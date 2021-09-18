use crate::comp_mgr_new::{
    Component_Manager, Component_Storage, Component_Storage_Interface, Component_Storage_Read,
    Component_Storage_Write,
};
use crate::ecs_world::Entity;
use anymap::any::UncheckedAnyExt;
use std::any::TypeId;
use std::collections::HashMap;

struct Ecs_Query<'mgr, 'str> {
    comp_mgr: &'mgr Component_Manager,
    storages: Storages<'str>,
    entities: Vec<Entity>,
}

#[derive(Default)]
struct Storages<'a> {
    reads: Vec<&'a dyn Component_Storage_Interface>,
    writes: Vec<&'a dyn Component_Storage_Interface>,

    read_indices: HashMap<TypeId, usize>,
    write_indices: HashMap<TypeId, usize>,
}

impl Storages<'_> {
    pub fn begin_read<T: 'static>(&self) -> Component_Storage_Read<T> {
        let idx = self.read_indices.get(&TypeId::of::<T>()).unwrap();
        let storage = unsafe { self.reads[*idx].downcast_ref_unchecked::<Component_Storage<T>>() };
        storage.lock_for_read()
    }

    pub fn begin_write<T: 'static>(&self) -> Component_Storage_Write<T> {
        let idx = self.write_indices.get(&TypeId::of::<T>()).unwrap();
        let storage = unsafe { self.writes[*idx].downcast_ref_unchecked::<Component_Storage<T>>() };
        storage.lock_for_write()
    }
}

impl<'mgr, 'str> Ecs_Query<'mgr, 'str>
where
    'mgr: 'str,
{
    // @Refactor: construct using Ecs_World rather than separate comp_mgr/entities
    fn new(comp_mgr: &'mgr Component_Manager, entities: &[Entity]) -> Self {
        Self {
            comp_mgr,
            storages: Storages::default(),
            entities: entities.to_vec(),
        }
    }

    fn read<T: 'static>(mut self) -> Self {
        let storage = self
            .comp_mgr
            .get_component_storage::<T>()
            .expect("TODO: handle missing comp");
        self.storages.reads.push(storage);
        self.storages
            .read_indices
            .insert(TypeId::of::<T>(), self.storages.reads.len() - 1);

        // @Speed: this may probably be accelerated with some dedicated data structure on Component_Manager
        let comp_mgr = self.comp_mgr;
        self.entities.retain(|&e| comp_mgr.has_component::<T>(e));

        self
    }

    fn write<T: 'static>(mut self) -> Self {
        let storage = self
            .comp_mgr
            .get_component_storage::<T>()
            .expect("TODO: handle missing comp");
        self.storages.writes.push(storage);
        self.storages
            .write_indices
            .insert(TypeId::of::<T>(), self.storages.writes.len() - 1);

        // @Speed: this may probably be accelerated with some dedicated data structure on Component_Manager
        let comp_mgr = self.comp_mgr;
        self.entities.retain(|&e| comp_mgr.has_component::<T>(e));

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecs_query() {
        struct A {
            x: u32,
        }
        struct B {
            y: f32,
        }
        struct C {
            s: String,
        }
        struct D {
            v: Vec<String>,
        }

        let mut comp_mgr = Component_Manager::new();
        let entities = [
            Entity { index: 1, gen: 1 },
            Entity { index: 2, gen: 1 },
            Entity { index: 3, gen: 1 },
        ];

        for e in &entities {
            comp_mgr.add_component(*e, A { x: 42 });
            comp_mgr.add_component(*e, B { y: 0.3 });
            comp_mgr.add_component(
                *e,
                C {
                    s: "Hello sailor".to_string(),
                },
            );
            comp_mgr.add_component(
                *e,
                D {
                    v: vec!["asd".to_string(), "bar".to_string()],
                },
            );
        }

        foreach_entity_new!(comp_mgr, entities,
            read: A, B;
            write: C, D;
            |entity, (a, b): (&A, &B), (c, d): (&mut C, &mut D)| {
            c.s = format!("{}, {}", a.x, b.y);
            d.v.push(c.s.clone());

            assert_eq!(d.v[d.v.len() - 1], "42, 0.3");
        });
    }
}
