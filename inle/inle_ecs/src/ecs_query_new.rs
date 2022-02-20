use crate::comp_mgr::{Component_Manager, Component_Type};
use crate::ecs_world::Entity;
use std::any::TypeId;
use std::collections::HashSet;

pub struct Ecs_Query {
    pub entities: Vec<Entity>,

    pub comp_read: HashSet<Component_Type>,
    pub comp_write: HashSet<Component_Type>,
}

impl Ecs_Query {
    pub fn new() -> Self {
        Ecs_Query {
            entities: vec![],
            comp_read: HashSet::default(),
            comp_write: HashSet::default(),
        }
    }

    pub fn read<T: 'static>(&mut self) {
        self.comp_read.insert(TypeId::of::<T>());
    }

    pub fn write<T: 'static>(&mut self) {
        self.comp_write.insert(TypeId::of::<T>());
    }

    pub fn update(
        &mut self,
        comp_mgr: &Component_Manager,
        entity: Entity,
        comp_added: &[Component_Type],
        comp_removed: &[Component_Type],
    ) {
        #[cfg(debug_assertions)]
        use std::iter::FromIterator;

        debug_assert!(HashSet::<&Component_Type>::from_iter(comp_added.iter())
            .is_disjoint(&HashSet::from_iter(comp_removed.iter())));

        if let Some(idx) = self.entities.iter().position(|&e| e == entity) {
            for comp in comp_removed {
                if self.comp_read.contains(comp) || self.comp_write.contains(comp) {
                    self.entities.swap_remove(idx);
                    return;
                }
            }
        } else if self
            .comp_read
            .iter()
            .chain(self.comp_write.iter())
            .all(|comp| comp_mgr.has_component_dyn(entity, comp))
        {
            self.entities.push(entity);
        }
    }
}

/*

A query contains the list of entities that have the required components.
It gets updated every time a component is added to /removed from an entity.

// create query
let qry = Ecs_Query::new()
    .read<C1>()
    .write<C2>();

// update query
qry.update(entity, comp_added, comp_removed);

// use query
foreach_entity!(ecs_world, qry, |e, (c1: &C1,), (c2: &mut C2,)| {
    ...
});

*/
