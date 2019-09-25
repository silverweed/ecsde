use crate::alloc::generational_allocator::{Generational_Allocator, Generational_Index};
use crate::ecs::components::transform::C_Transform2D;
use crate::ecs::entity_manager::Entity;
use std::vec::Vec;

type Node = Generational_Index;

pub struct Scene_Tree {
    node_allocator: Generational_Allocator,

    // Associates entity_id => node_id.
    // Index in the array is the entity index.
    nodes: Vec<Option<Node>>,

    // Associates node_id => parent_node_id.
    // Index in the array is the child node index.
    // First slot, the one associated to the root node, has a different meaning:
    // it's 0 if the root has not been set, 1 otherwise.
    hierarchy: Vec<Node>,

    local_transforms: Vec<C_Transform2D>,
    global_transforms: Vec<C_Transform2D>,
}

impl Scene_Tree {
    pub fn new() -> Scene_Tree {
        Scene_Tree {
            node_allocator: Generational_Allocator::new(32),
            nodes: vec![],
            hierarchy: vec![Generational_Index { index: 0, gen: 0 }],
            local_transforms: vec![],
            global_transforms: vec![],
        }
    }

    pub fn add(&mut self, e: Entity, parent: Option<Entity>, local_transform: &C_Transform2D) {
        // Check invariants
        assert_eq!(self.local_transforms.len(), self.global_transforms.len());
        assert!(self.local_transforms.len() <= self.hierarchy.len());
        assert!(!self.hierarchy.is_empty());

        // Ensure we have enough space in the entity => node map
        if e.index >= self.nodes.len() {
            self.nodes.resize(e.index + 1, None);
        }

        // Associate this entity with a new node
        let mut child_node = self.node_allocator.allocate();

        if let Some(parent) = parent {
            if e == parent {
                println!(
                    "[ ERROR ] Tried to add entity {:?} as a child of itself.",
                    e
                );
                return;
            }

            if parent.index >= self.nodes.len() {
                println!(
                    "[ ERROR ] Invalid parent {:?} when adding {:?} to scene tree",
                    parent, e
                );
                return;
            }

            let parent_node = self.nodes[parent.index as usize];
            if parent_node.is_none() {
                println!(
                    "[ ERROR ] Invalid parent {:?} when adding {:?} to scene tree",
                    parent, e
                );
                return;
            }

            let mut parent_node = parent_node.unwrap();

            if !self.node_allocator.is_valid(parent_node) {
                println!(
                    "[ ERROR ] Parent {:?} was already expired when adding {:?} to scene tree",
                    parent, e
                );
                return;
            }

            assert_ne!(child_node.index, parent_node.index);

            // Ensure the child node's id is greater than the parent's. If that's not the case, swap their node ids.
            if child_node.index < parent.index {
                assert_eq!(self.nodes[child_node.index as usize], None);
                self.nodes[parent.index as usize] = Some(child_node);
                for i in 0..self.hierarchy.len() {
                    if self.hierarchy[i] == parent_node {
                        self.hierarchy[i] = child_node;
                    }
                }
                std::mem::swap(&mut parent_node, &mut child_node);
            }

            // Add the new node to the list along with its parent and its associated transform
            self.nodes[e.index as usize] = Some(child_node);
            if child_node.index >= self.hierarchy.len() {
                assert_eq!(self.hierarchy.len(), child_node.index - 1);
                self.hierarchy.push(parent_node);
                self.local_transforms.push(*local_transform);
                self.global_transforms.push(*local_transform);
            } else {
                self.hierarchy[child_node.index as usize - 1] = parent_node;
                self.local_transforms[child_node.index as usize - 1] = *local_transform;
                self.global_transforms[child_node.index as usize - 1] = *local_transform;
            }
        } else {
            // This entity is the root.
            if self.hierarchy[0].index != 0 {
                println!("[ WARNING ] Overriding the root in a Scene_Tree");
            }
            self.nodes[e.index] = Some(self.node_allocator.allocate());
            self.hierarchy[0] = Generational_Index { index: 1, gen: 0 };
            self.local_transforms.push(*local_transform);
            self.global_transforms.push(*local_transform);
        }

        assert_eq!(self.local_transforms.len(), self.hierarchy.len());
    }

    pub fn set_local_transform(&mut self, e: Entity, new_transform: &C_Transform2D) {
        self.local_transforms[self.nodes[e.index as usize].unwrap().index as usize - 1] =
            *new_transform;
    }

    pub fn get_global_transform(&self, e: Entity) -> Option<&C_Transform2D> {
        self.global_transforms
            .get(self.nodes.get(e.index as usize)?.unwrap().index as usize - 1)
    }

    pub fn compute_global_transforms(&mut self) {
        let local_transforms = &self.local_transforms;
        let global_transforms = &mut self.global_transforms;
        let hierarchy = &self.hierarchy;

        // Root has no parent
        global_transforms[0] = local_transforms[0];

        for i in 1..global_transforms.len() {
            let parent_index = hierarchy[i].index as usize - 1;
            // @Speed: this recalculates matrices every time! Optimize this!
            global_transforms[i] = C_Transform2D::new_from_matrix(
                &(global_transforms[parent_index].get_matrix() * local_transforms[i].get_matrix()),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::entity_manager::Entity_Manager;
    use cgmath::{Deg, Matrix3, Rad};
    use float_cmp::ApproxEq;
    use std::convert::Into;

    #[test]
    fn simple_tree() {
        let mut em = Entity_Manager::new();
        em.register_component::<C_Transform2D>();

        let mut tree = Scene_Tree::new();

        let root_e = em.new_entity();
        {
            let root_t = em.add_component::<C_Transform2D>(root_e);
            tree.add(root_e, None, &root_t);
        }
        let child_e = em.new_entity();
        {
            let mut child_t = em.add_component::<C_Transform2D>(child_e);
            child_t.set_position(100.0, 0.0);
            tree.add(child_e, Some(root_e), &child_t);
        }

        tree.compute_global_transforms();

        let new_child_t = tree.get_global_transform(child_e).unwrap();
        assert!(new_child_t.position().x.approx_eq(100.0, (0.0, 2)));

        let mut root_t = em.get_component_mut::<C_Transform2D>(root_e).unwrap();
        root_t.rotate(Deg(90.0));
        tree.set_local_transform(root_e, &root_t);

        tree.compute_global_transforms();

        let new_child_t = tree.get_global_transform(child_e).unwrap();
        assert!(new_child_t.position().y.approx_eq(100.0, (0.0, 2)));
    }
}
