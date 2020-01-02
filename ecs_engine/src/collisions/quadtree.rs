use super::collider::{Collider, Collider_Shape};
use crate::core::common::rect::{Rect, Rectf};
use crate::core::common::transform::Transform2D;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::ecs_world::{Ecs_World, Entity};

const MAX_OBJECTS: usize = 8;

pub struct Quad_Tree {
    bounds: Rectf,
    objects: Vec<Entity>,
    subnodes: Option<[Box<Quad_Tree>; 4]>,
}

impl Quad_Tree {
    pub fn new(bounds: Rectf) -> Self {
        Quad_Tree {
            bounds,
            objects: vec![],
            subnodes: None,
        }
    }

    pub fn clear(&mut self) {
        self.subnodes = None;
        self.objects.clear();
    }

    pub fn add(
        &mut self,
        entity: Entity,
        collider: &Collider,
        transform: &Transform2D,
        ecs_world: &Ecs_World,
    ) {
        if let Some(subnodes) = &mut self.subnodes {
            let index = get_index(collider, transform, &self.bounds);
            if index >= 0 {
                subnodes[index as usize].add(entity, collider, transform, ecs_world);
            }
        } else {
            self.objects.push(entity);
            // @Audit: should we add a MAX_DEPTH to the tree?
            if self.objects.len() > MAX_OBJECTS {
                if self.subnodes.is_none() {
                    self.split();
                }

                let mut i = 0;
                let subnodes = self.subnodes.as_mut().unwrap();
                while i < self.objects.len() {
                    let entity = self.objects[i];
                    let collider = ecs_world.get_component::<Collider>(entity).unwrap();
                    let transform = &ecs_world
                        .get_component::<C_Spatial2D>(entity)
                        .unwrap()
                        .global_transform;
                    let index = get_index(collider, transform, &self.bounds);
                    if index >= 0 {
                        subnodes[index as usize].add(
                            self.objects.swap_remove(i),
                            collider,
                            transform,
                            ecs_world,
                        );
                    } else {
                        i += 1;
                    }
                }
            }
        }
    }

    pub fn get_neighbours(
        &self,
        collider: &Collider,
        transform: &Transform2D,
        result: &mut Vec<Entity>,
    ) {
        if let Some(subnodes) = &self.subnodes {
            let index = get_index(collider, transform, &self.bounds);
            if index >= 0 {
                subnodes[index as usize].get_neighbours(collider, transform, result);
            }
        }

        for obj in &self.objects {
            result.push(*obj);
        }
    }

    fn split(&mut self) {
        let bounds = &self.bounds;
        let subw = bounds.width() * 0.5;
        let subh = bounds.height() * 0.5;
        let x = bounds.x();
        let y = bounds.y();

        self.subnodes = Some([
            Box::new(Quad_Tree::new(Rect::new(x, y, subw, subh))),
            Box::new(Quad_Tree::new(Rect::new(x + subw, y, subw, subh))),
            Box::new(Quad_Tree::new(Rect::new(x, y + subh, subw, subh))),
            Box::new(Quad_Tree::new(Rect::new(x + subw, y + subh, subw, subh))),
        ]);
    }
}

fn get_index(collider: &Collider, transform: &Transform2D, bounds: &Rectf) -> i8 {
    use crate::core::common::vector::Vec2f;

    let mut idx = -1;
    let horiz_mid = bounds.x() + bounds.width() * 0.5;
    let vert_mid = bounds.y() + bounds.height() * 0.5;
    let Vec2f { x: obj_x, y: obj_y } = transform.position();
    let Vec2f {
        x: obj_scale_x,
        y: obj_scale_y,
    } = transform.scale();

    let fits_top;
    let fits_bot;
    let fits_left;
    let fits_right;
    // @Incomplete: we're not using the rotation!
    match collider.shape {
        Collider_Shape::Rect { width, height } => {
            let width = width * obj_scale_x;
            let height = height * obj_scale_y;
            fits_top = obj_y > bounds.y() && obj_y + height < vert_mid;
            fits_bot = obj_y > vert_mid && obj_y + height < bounds.y() + bounds.height();
            fits_left = obj_x > bounds.x() && obj_x + width < horiz_mid;
            fits_right = obj_x > horiz_mid && obj_x + width < bounds.x() + bounds.width();
        }
    }

    debug_assert!(!(fits_top && fits_bot));
    debug_assert!(!(fits_left && fits_right));

    if fits_left {
        if fits_top {
            idx = 0;
        } else if fits_bot {
            idx = 2;
        }
    } else if fits_right {
        if fits_top {
            idx = 1;
        } else if fits_bot {
            idx = 3;
        }
    }

    idx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::common::transform::Transform2D;
    use crate::core::common::vector::Vec2f;
    use cgmath::Rad;

    fn depth(quadtree: &Quad_Tree) -> usize {
        fn depth_internal(quadtree: &Quad_Tree, val: usize) -> usize {
            if let Some(subnodes) = &quadtree.subnodes {
                subnodes
                    .iter()
                    .map(|node| depth_internal(node, val + 1))
                    .max()
                    .unwrap()
            } else {
                val
            }
        }
        depth_internal(quadtree, 1)
    }

    #[test]
    fn test_quadtree() {
        let mut tree = Quad_Tree::new(Rect::new(-100., -100., 200., 200.));
        let mut ecs_world = Ecs_World::new();
        ecs_world.register_component::<Collider>();
        ecs_world.register_component::<C_Spatial2D>();

        fn cld(
            ecs_world: &mut Ecs_World,
            (x, y): (f32, f32),
            (sx, sy): (f32, f32),
            (w, h): (f32, f32),
        ) -> (Entity, Collider, Transform2D) {
            let entity = ecs_world.new_entity();
            let cld = ecs_world.add_component::<Collider>(entity);
            let collider = Collider {
                shape: Collider_Shape::Rect {
                    width: w,
                    height: h,
                },
            };
            *cld = collider;

            let spat = ecs_world.add_component::<C_Spatial2D>(entity);
            let trans =
                Transform2D::from_pos_rot_scale(Vec2f::new(x, y), Rad(0.), Vec2f::new(sx, sy));
            spat.global_transform = trans;

            (entity, collider, trans)
        }

        let mut results = vec![];

        let (e1, c1, t1) = cld(&mut ecs_world, (0., 0.), (1., 1.), (10., 10.));
        tree.add(e1, &c1, &t1, &ecs_world);

        tree.get_neighbours(&c1, &t1, &mut results);
        assert_eq!(results.len(), 1);
        assert_eq!(depth(&tree), 1);

        let (e2, c2, t2) = cld(&mut ecs_world, (50., 0.), (2., 2.), (3., 3.));
        tree.add(e2, &c2, &t2, &ecs_world);

        let (e3, c3, t3) = cld(&mut ecs_world, (-35., -70.), (1.5, 1.5), (2.5, 2.5));
        tree.add(e3, &c3, &t3, &ecs_world);

        results.clear();
        tree.get_neighbours(&c1, &t1, &mut results);
        assert_eq!(results.len(), 3);
        assert_eq!(depth(&tree), 1);

        // Check the tree splits after MAX_OBJECTS adds
        for _ in 3..=MAX_OBJECTS {
            tree.add(e3, &c3, &t3, &ecs_world);
        }

        assert_eq!(depth(&tree), 2);

        results.clear();
        tree.get_neighbours(&c2, &t2, &mut results);
        // All c3's should not be in the neighbour list.
        assert_eq!(results.len(), 2);

        tree.clear();
        assert_eq!(depth(&tree), 1);
    }
}
