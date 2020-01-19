use super::collider::{Collider, Collider_Shape};
use crate::core::common::rect::{Rect, Rectf};
use crate::core::common::transform::Transform2D;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::ecs_world::{Ecs_World, Entity};
use crate::prelude::*;

#[cfg(debug_assertions)]
use crate::debug::debug_painter::Debug_Painter;

const MAX_OBJECTS: usize = 8;
const MAX_DEPTH: u8 = 10;

pub struct Quad_Tree {
    bounds: Rectf,
    objects: Vec<Entity>,
    subnodes: Option<[Box<Quad_Tree>; 4]>,
    /// level of nesting of this tree
    level: u8,
}

impl Quad_Tree {
    pub fn new(bounds: Rectf) -> Self {
        Quad_Tree {
            bounds,
            objects: vec![],
            subnodes: None,
            level: 0,
        }
    }

    fn new_nested(bounds: Rectf, parent: &Quad_Tree) -> Self {
        Quad_Tree {
            bounds,
            objects: vec![],
            subnodes: None,
            level: parent.level + 1,
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
        tracer: Debug_Tracer,
    ) {
        if let Some(subnodes) = &mut self.subnodes {
            let index = get_index(collider, transform, &self.bounds);
            if index >= 0 {
                subnodes[index as usize].add(
                    entity,
                    collider,
                    transform,
                    ecs_world,
                    clone_tracer!(tracer),
                );
            }
        } else {
            self.objects.push(entity);
            if self.objects.len() > MAX_OBJECTS && self.level < MAX_DEPTH {
                if self.subnodes.is_none() {
                    trace!("quadtree::split", tracer);
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
                            clone_tracer!(tracer),
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
            Box::new(Quad_Tree::new_nested(Rect::new(x, y, subw, subh), &self)),
            Box::new(Quad_Tree::new_nested(
                Rect::new(x + subw, y, subw, subh),
                &self,
            )),
            Box::new(Quad_Tree::new_nested(
                Rect::new(x, y + subh, subw, subh),
                &self,
            )),
            Box::new(Quad_Tree::new_nested(
                Rect::new(x + subw, y + subh, subw, subh),
                &self,
            )),
        ]);
    }
}

fn get_index(collider: &Collider, transform: &Transform2D, bounds: &Rectf) -> i8 {
    use crate::core::common::vector::Vec2f;

    let mut idx = -1;
    let horiz_mid = bounds.x() + bounds.width() * 0.5;
    let vert_mid = bounds.y() + bounds.height() * 0.5;
    let Vec2f { x: obj_x, y: obj_y } = transform.position() + collider.offset;
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

#[cfg(debug_assertions)]
pub(super) fn draw_quadtree(quadtree: &Quad_Tree, painter: &mut Debug_Painter) {
    use crate::core::common::colors;
    use crate::core::common::vector::Vec2f;
    use crate::gfx::render;

    fn calc_quadtree_deepth(quadtree: &Quad_Tree) -> u32 {
        fn calc_quadtree_deepth_internal(quadtree: &Quad_Tree, depth: u32) -> u32 {
            if let Some(subnodes) = &quadtree.subnodes {
                subnodes
                    .iter()
                    .map(|subnode| calc_quadtree_deepth_internal(subnode, depth + 1))
                    .max()
                    .unwrap_or(depth)
            } else {
                depth
            }
        }

        calc_quadtree_deepth_internal(quadtree, 1)
    }

    let depth = calc_quadtree_deepth(quadtree);

    fn draw_quadtree_internal(quadtree: &Quad_Tree, painter: &mut Debug_Painter, depth: u32) {
        assert!(
            depth > quadtree.level as u32,
            "quadtree.level >= quadtree.depth! ({} >= {})",
            quadtree.level,
            depth
        );
        let props = render::Paint_Properties {
            color: colors::rgba(102, 204, 255, 20),
            border_thick: (((depth - quadtree.level as u32) * 2) as f32).max(1.),
            border_color: colors::rgba(255, 0, 255, 150),
            ..Default::default()
        };
        let transform = Transform2D::from_pos(Vec2f::new(quadtree.bounds.x(), quadtree.bounds.y()));

        painter.add_rect(quadtree.bounds.size(), &transform, &props);
        if let Some(subnodes) = &quadtree.subnodes {
            for subnode in subnodes {
                draw_quadtree_internal(subnode, painter, depth);
            }
        }
    }

    draw_quadtree_internal(quadtree, painter, depth);
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
                ..Default::default()
            };
            *cld = collider;

            let spat = ecs_world.add_component::<C_Spatial2D>(entity);
            let trans =
                Transform2D::from_pos_rot_scale(Vec2f::new(x, y), Rad(0.), Vec2f::new(sx, sy));
            spat.global_transform = trans;

            (entity, collider, trans)
        }

        let mut results = vec![];
        let tracer = new_debug_tracer();

        let (e1, c1, t1) = cld(&mut ecs_world, (0., 0.), (1., 1.), (10., 10.));
        tree.add(e1, &c1, &t1, &ecs_world, clone_tracer!(tracer));

        tree.get_neighbours(&c1, &t1, &mut results);
        assert_eq!(results.len(), 1);
        assert_eq!(depth(&tree), 1);

        let (e2, c2, t2) = cld(&mut ecs_world, (50., 0.), (2., 2.), (3., 3.));
        tree.add(e2, &c2, &t2, &ecs_world, clone_tracer!(tracer));

        let (e3, c3, t3) = cld(&mut ecs_world, (-35., -70.), (1.5, 1.5), (2.5, 2.5));
        tree.add(e3, &c3, &t3, &ecs_world, clone_tracer!(tracer));

        results.clear();
        tree.get_neighbours(&c1, &t1, &mut results);
        assert_eq!(results.len(), 3);
        assert_eq!(depth(&tree), 1);

        // Check the tree splits after MAX_OBJECTS adds
        for _ in 3..=MAX_OBJECTS {
            tree.add(e3, &c3, &t3, &ecs_world, clone_tracer!(tracer));
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
