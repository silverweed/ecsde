use super::collider::Collider;
use super::quadtree::Quad_Tree;
use crate::core::common::rect::Rect;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::ecs_world::Ecs_World;
use crate::ecs::entity_stream::new_entity_stream;

pub struct Collision_System {
    quadtree: Quad_Tree,
}

impl Collision_System {
    pub fn new() -> Self {
        // @Incomplete
        let world_rect = Rect::new(-100000., -100000., 200000., 200000.);
        Collision_System {
            quadtree: Quad_Tree::new(world_rect),
        }
    }

    pub fn update(&mut self, ecs_world: &Ecs_World) {
        self.quadtree.clear();

        let mut stream = new_entity_stream(ecs_world)
            .require::<Collider>()
            .require::<C_Spatial2D>()
            .build();
        loop {
            let entity = stream.next(ecs_world);
            if entity.is_none() {
                break;
            }
            let entity = entity.unwrap();

            let collider = ecs_world.get_component::<Collider>(entity).unwrap();
            let transform = &ecs_world
                .get_component::<C_Spatial2D>(entity)
                .unwrap()
                .global_transform;

            self.quadtree.add(entity, collider, transform, ecs_world);
        }

        // @Incomplete: add collision detection
    }
}
