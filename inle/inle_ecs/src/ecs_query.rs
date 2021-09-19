use crate::comp_mgr::{
    Component_Manager, Component_Storage, Component_Storage_Interface, Component_Storage_Read,
    Component_Storage_Write,
};
use crate::ecs_world::{Ecs_World, Entity};
use anymap::any::UncheckedAnyExt;
use std::any::{type_name, TypeId};
use std::collections::HashMap;

pub struct Ecs_Query<'mgr, 'str> {
    comp_mgr: &'mgr Component_Manager,
    storages: Storages<'str>,
    entities: Vec<Entity>,
}

impl<'m, 's> Ecs_Query<'m, 's> {
    #[inline]
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    #[inline]
    pub fn storages(&self) -> &Storages {
        &self.storages
    }
}

#[derive(Default)]
pub struct Storages<'a> {
    reads: Vec<&'a dyn Component_Storage_Interface>,
    writes: Vec<&'a dyn Component_Storage_Interface>,

    read_indices: HashMap<TypeId, usize>,
    write_indices: HashMap<TypeId, usize>,

    #[cfg(debug_assertions)]
    human_readable_query: String,
}

impl Storages<'_> {
    pub fn begin_read<T: 'static>(&self) -> Component_Storage_Read<T> {
        trace!("ecs_query::storages::begin_read");

        let idx = self
            .read_indices
            .get(&TypeId::of::<T>())
            .unwrap_or_else(|| {
                let mut_in_debug!(msg) = format!(
                    "Tried to read type {:?} but either it is not part of this query or there are no entities satisfying this query. You should always check if query.entities() is empty before calling begin_read() or begin_write().",
                    base_type_name::<T>()
                );
                #[cfg(debug_assertions)]
                {
                    msg.push_str(&format!(" (query = {})", self.human_readable_query));
                }
                panic!("{}", msg);
            });
        let storage = unsafe { self.reads[*idx].downcast_ref_unchecked::<Component_Storage<T>>() };
        storage.lock_for_read()
    }

    pub fn begin_write<T: 'static>(&self) -> Component_Storage_Write<T> {
        trace!("ecs_query::storages::begin_write");

        let idx = self
            .write_indices
            .get(&TypeId::of::<T>())
            .unwrap_or_else(|| {
                let mut_in_debug!(msg) = format!(
                    "Tried to write type {:?} but either it is not part of this query or there are no entities satisfying this query. You should always check if query.entities() is empty before calling begin_read() or begin_write().",
                    base_type_name::<T>()
                );
                #[cfg(debug_assertions)]
                {
                    msg.push_str(&format!(" (query = {})", self.human_readable_query));
                }
                panic!("{}", msg);
            });
        let storage = unsafe { self.writes[*idx].downcast_ref_unchecked::<Component_Storage<T>>() };
        storage.lock_for_write()
    }
}

impl<'mgr, 'str> Ecs_Query<'mgr, 'str>
where
    'mgr: 'str,
{
    pub fn new(ecs_world: &'mgr Ecs_World) -> Self {
        Self {
            comp_mgr: &ecs_world.component_manager,
            storages: Storages::default(),
            entities: ecs_world.entities().to_vec(),
        }
    }

    pub fn read<T: 'static>(mut self) -> Self {
        trace!("ecs_query::read");

        #[cfg(debug_assertions)]
        {
            self.storages
                .human_readable_query
                .push_str(&format!("read {}, ", base_type_name::<T>()));
        }

        let mut added = false;
        if !self.entities.is_empty() {
            if let Some(storage) = self.comp_mgr.get_component_storage::<T>() {
                self.storages.reads.push(storage);
                self.storages
                    .read_indices
                    .insert(TypeId::of::<T>(), self.storages.reads.len() - 1);

                // @Speed: this may probably be accelerated with some dedicated data structure on Component_Manager
                let comp_mgr = self.comp_mgr;
                {
                    trace!("ecs_query::entities_retain");
                    self.entities.retain(|&e| comp_mgr.has_component::<T>(e));
                }

                added = true;
            }
        }

        if !added {
            // Instead of retaining, since we know no entity has this component, just clear the array.
            self.entities.clear();
        }

        self
    }

    pub fn write<T: 'static>(mut self) -> Self {
        trace!("ecs_query::write");

        #[cfg(debug_assertions)]
        {
            self.storages
                .human_readable_query
                .push_str(&format!("write {}, ", base_type_name::<T>()));
        }

        let mut added = false;
        if !self.entities.is_empty() {
            if let Some(storage) = self.comp_mgr.get_component_storage::<T>() {
                self.storages.writes.push(storage);
                self.storages
                    .write_indices
                    .insert(TypeId::of::<T>(), self.storages.writes.len() - 1);

                // @Speed: this may probably be accelerated with some dedicated data structure on Component_Manager
                let comp_mgr = self.comp_mgr;
                {
                    trace!("ecs_query::entities_retain");
                    self.entities.retain(|&e| comp_mgr.has_component::<T>(e));
                }

                added = true;
            }
        }

        if !added {
            self.entities.clear();
        }

        self
    }
}

#[cfg(debug_assertions)]
fn base_type_name<T>() -> &'static str {
    let full_name = type_name::<T>();
    let base_name = full_name.rsplit(':').next().unwrap_or(full_name);
    base_name
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

        let mut world = Ecs_World::new();
        let entities = [world.new_entity(), world.new_entity(), world.new_entity()];

        for e in &entities {
            world.add_component(*e, A { x: 42 });
            world.add_component(*e, B { y: 0.3 });
            world.add_component(
                *e,
                C {
                    s: "Hello sailor".to_string(),
                },
            );
            world.add_component(
                *e,
                D {
                    v: vec!["asd".to_string(), "bar".to_string()],
                },
            );
        }

        foreach_entity!(&world,
            read: A, B;
            write: C, D;
            |_entity, (a, b): (&A, &B), (c, d): (&mut C, &mut D)| {
            c.s = format!("{}, {}", a.x, b.y);
            d.v.push(c.s.clone());

            assert_eq!(d.v[d.v.len() - 1], "42, 0.3");
        });
    }
}
