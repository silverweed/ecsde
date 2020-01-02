use super::collider::{Collider, Collider_Shape};
use super::quadtree::Quad_Tree;
use crate::core::common::rect::{self, Rect};
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::ecs_world::{Ecs_World, Entity};
use crate::ecs::entity_stream::new_entity_stream;
use crate::prelude::*;
use crossbeam::thread;

pub struct Collision_System {
    quadtree: Quad_Tree,
    entities_buf: Vec<Entity>,
}

impl Collision_System {
    pub fn new() -> Self {
        // @Incomplete
        let world_rect = Rect::new(-100000., -100000., 200000., 200000.);
        Collision_System {
            quadtree: Quad_Tree::new(world_rect),
            entities_buf: vec![],
        }
    }

    pub fn update(&mut self, ecs_world: &mut Ecs_World, tracer: Debug_Tracer) {
        // Step 1: fill quadtree
        self.quadtree.clear();

        {
            trace!("collision_system::fill_quadtree", tracer);
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

                let collider = {
                    let collider = ecs_world.get_component_mut::<Collider>(entity).unwrap();
                    collider.colliding = false;
                    collider.clone()
                };

                let transform = &ecs_world
                    .get_component::<C_Spatial2D>(entity)
                    .unwrap()
                    .global_transform;

                self.quadtree.add(entity, &collider, transform, ecs_world);
            }
        }

        // Step 2: do collision detection

        {
            trace!("collision_system::collision_detection", tracer);

            // @Refactor: can we avoid doing all the queries twice?
            let mut stream = new_entity_stream(ecs_world)
                .require::<Collider>()
                .require::<C_Spatial2D>()
                .build();
            self.entities_buf.clear();
            loop {
                let entity = stream.next(ecs_world);
                if entity.is_none() {
                    break;
                }
                self.entities_buf.push(entity.unwrap());
            }

            use std::sync::atomic::AtomicUsize;
            use std::sync::{Arc, Mutex};
            let n_collisions_total = Arc::new(AtomicUsize::new(0));
            let n_entities = self.entities_buf.len();
            let collided_entities = Arc::new(Mutex::new(vec![]));

            thread::scope(|s| {
                let n_threads = num_cpus::get();
                for ent_chunk in self.entities_buf.chunks(n_entities / n_threads + 1) {
                    let quadtree = &self.quadtree;
                    let n_collisions_total = n_collisions_total.clone();
                    let collided_entities = collided_entities.clone();
                    let ecs_world = ecs_world as &Ecs_World;
                    s.spawn(move |_| {
                        for entity in ent_chunk.into_iter() {
                            let collider = ecs_world.get_component::<Collider>(*entity).unwrap();
                            let transform = &ecs_world
                                .get_component::<C_Spatial2D>(*entity)
                                .unwrap()
                                .global_transform;
                            let pos = transform.position();
                            let scale = transform.scale();

                            let mut neighbours = vec![];
                            quadtree.get_neighbours(collider, transform, &mut neighbours);
                            if !neighbours.is_empty() {
                                // Check collision with neighbours
                                for neighbour in &neighbours {
                                    let oth_cld =
                                        ecs_world.get_component::<Collider>(*neighbour).unwrap();
                                    let oth_transf = &ecs_world
                                        .get_component::<C_Spatial2D>(*neighbour)
                                        .unwrap()
                                        .global_transform;
                                    let oth_pos = oth_transf.position();
                                    let oth_scale = oth_transf.scale();

                                    match collider.shape {
                                        Collider_Shape::Rect { width, height } => match oth_cld
                                            .shape
                                        {
                                            Collider_Shape::Rect {
                                                width: oth_width,
                                                height: oth_height,
                                            } => {
                                                let me = Rect::new(
                                                    pos.x,
                                                    pos.y,
                                                    width * scale.x,
                                                    height * scale.y,
                                                );
                                                let him = Rect::new(
                                                    oth_pos.x,
                                                    oth_pos.y,
                                                    oth_width * oth_scale.x,
                                                    oth_height * oth_scale.y,
                                                );
                                                if rect::rects_intersect(&me, &him) {
                                                    if let Ok(mut cld) = collided_entities.lock() {
                                                        cld.push(entity);
                                                    }
                                                    n_collisions_total.fetch_add(
                                                        neighbours.len(),
                                                        std::sync::atomic::Ordering::Relaxed,
                                                    );
                                                }
                                            }
                                        },
                                    }
                                }
                            }
                        }
                    });
                }
            })
            .unwrap();

            if let Ok(cld) = collided_entities.lock() {
                for entity in cld.iter() {
                    let collider = ecs_world.get_component_mut::<Collider>(**entity).unwrap();
                    collider.colliding = true;
                }
            }

            println!(
                "tot collisions: {}, average: {}",
                n_collisions_total.load(std::sync::atomic::Ordering::SeqCst),
                n_collisions_total.load(std::sync::atomic::Ordering::SeqCst) / n_entities
            );
        }
    }
}
