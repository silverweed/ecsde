use crate::ecs::components::transform::C_Transform2D;
use crate::ecs::entity_manager::Entity;
use std::vec::Vec;

pub struct Scene_Tree {
    hierarchy: Vec<u32>,
    gens: Vec<u32>,
    local_transforms: Vec<C_Transform2D>,
    global_transforms: Vec<C_Transform2D>,
}

impl Scene_Tree {
    pub fn new() -> Scene_Tree {
        Scene_Tree {
            hierarchy: vec![],
            gens: vec![],
            local_transforms: vec![],
            global_transforms: vec![],
        }
    }

    pub fn add(&mut self, e: Entity, parent: Option<Entity>, local_transform: &C_Transform2D) {
        assert_eq!(self.local_transforms.len(), self.global_transforms.len());
        assert_eq!(self.local_transforms.len(), self.hierarchy.len());
        assert_eq!(self.local_transforms.len(), self.gens.len());

        if e.index >= self.global_transforms.len() {
            self.local_transforms
                .resize(e.index + 1, C_Transform2D::default());
            self.global_transforms
                .resize(e.index + 1, C_Transform2D::default());
            self.hierarchy.resize(e.index + 1, 0);
            self.gens.resize(e.index + 1, 0);
        }
        self.local_transforms[e.index] = transform;
        self.gens[e.index] = e.gen;

        if let Some(parent) = parent {
            assert_eq!(parent.gen, self.gens[parent.index]);
            self.hierarchy[e.index] = parent.index;
        }
    }

    pub fn compute_global_transforms(&mut self) {
        let local_transforms = &self.local_transforms;
        let global_transforms = &mut self.global_transforms;
        let hierarchy = &self.hierarchy;

        // Root has no parent
        global_transforms[0] = local_transforms[0];

        for i in 1..global_transforms.len() {
            let parent = hierarchy[i];
            // TODO: this recalculates matrices every time! Optimize this!
            global_transforms[i] =
                local_transforms[parent].get_matrix() * local_transforms[i].get_matrix();
        }
    }
}
