use crate::comp_mgr::{Component_Manager, Component_Type};
use crate::ecs_world::Entity;
use std::collections::HashSet;

// A structure that keeps track of which entities have a specific set of components.
#[derive(Default, Debug)]
pub struct Ecs_Query {
    entities: Vec<Entity>,

    components: HashSet<Component_Type>,
}

pub enum Update_Result {
    Ignored,
    Added,
    Removed,
}

impl Ecs_Query {
    pub fn require<T: 'static>(mut self) -> Self {
        self.components.insert(Component_Type::create::<T>());
        self
    }

    pub fn update(
        &mut self,
        comp_mgr: &Component_Manager,
        entity: Entity,
        comp_added: &[Component_Type],
        comp_removed: &[Component_Type],
    ) -> Update_Result {
        #[cfg(debug_assertions)]
        {
            use std::iter::FromIterator;
            debug_assert!(HashSet::<&Component_Type>::from_iter(comp_added.iter())
                .is_disjoint(&HashSet::from_iter(comp_removed.iter())));
        }

        if let Some(idx) = self.entities.iter().position(|&e| e == entity) {
            for comp in comp_removed {
                if self.components.contains(comp) {
                    self.entities.swap_remove(idx);
                    return Update_Result::Removed;
                }
            }
        } else if comp_added
            .iter()
            .any(|comp| self.components.is_empty() || self.components.contains(comp))
            && self
                .components
                .iter()
                .chain(self.components.iter())
                .all(|comp| comp_mgr.has_component_dyn(entity, comp))
        {
            self.entities.push(entity);
            return Update_Result::Added;
        }

        Update_Result::Ignored
    }

    #[inline(always)]
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }
}
