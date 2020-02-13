use super::collider::{Collider, Collider_Shape};
use super::quadtree;
use crate::core::common::rect::{self, Rect};
use crate::core::common::shapes;
use crate::core::common::transform::Transform2D;
use crate::core::common::vector::Vec2f;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::ecs_world::{Components_Map_Safe, Ecs_World, Entity};
use crate::ecs::entity_stream::new_entity_stream;
use crate::prelude::*;
use crossbeam::thread;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

#[cfg(debug_assertions)]
use crate::debug::debug_painter::Debug_Painter;

#[derive(Clone, Debug)]
struct Collision_Info {
    pub my_pos: Vec2f,
    pub my_velocity: Vec2f,
    pub other: Entity,
    pub oth_pos: Vec2f,
    pub oth_velocity: Vec2f,
    pub penetration_dist: f32,
}

#[cfg(debug_assertions)]
struct Debug_Applied_Impulse {
    pub center: Vec2f,
    pub impulse: Vec2f,
}

pub struct Collision_System {
    quadtree: quadtree::Quad_Tree,
    entities_buf: Vec<Entity>,
    collided_entities: Arc<Mutex<HashMap<Entity, Collision_Info>>>,

    #[cfg(debug_assertions)]
    debug_applied_impulses: Vec<Debug_Applied_Impulse>,
}

impl Collision_System {
    pub fn new() -> Self {
        // @Incomplete
        let world_rect = Rect::new(-100_000., -100_000., 200_000., 200_000.);
        Collision_System {
            quadtree: quadtree::Quad_Tree::new(world_rect),
            entities_buf: vec![],
            collided_entities: Arc::new(Mutex::new(HashMap::new())),
            #[cfg(debug_assertions)]
            debug_applied_impulses: vec![],
        }
    }

    #[cfg(debug_assertions)]
    pub fn debug_draw_quadtree(&self, painter: &mut Debug_Painter) {
        quadtree::draw_quadtree(&self.quadtree, painter);
    }

    #[cfg(debug_assertions)]
    pub fn debug_draw_applied_impulses(&self, painter: &mut Debug_Painter) {
        for impulse in &self.debug_applied_impulses {
            painter.add_arrow(
                crate::core::common::shapes::Arrow {
                    center: impulse.center,
                    direction: impulse.impulse,
                    thickness: 3.,
                    arrow_size: 30.,
                },
                crate::core::common::colors::rgb(0, 153, 255),
            );
        }
    }

    #[cfg(debug_assertions)]
    pub fn debug_draw_entities_quad_id(&self, ecs_world: &Ecs_World, painter: &mut Debug_Painter) {
        use crate::core::common::colors;

        for &entity in &self.entities_buf {
            let id = self
                .quadtree
                .debug_get_quad_id(entity)
                .unwrap_or_else(|| String::from("None"));

            let transform = &ecs_world
                .get_component::<C_Spatial2D>(entity)
                .unwrap()
                .global_transform;

            painter.add_text(&id, transform.position(), 16, colors::rgb(0, 50, 200));
        }
    }

    pub fn update(&mut self, ecs_world: &mut Ecs_World, _tracer: Debug_Tracer) {
        // Step 1: fill quadtree
        {
            trace!("collision_system::clear_quadtree", _tracer);
            self.quadtree.clear();
        }

        #[cfg(debug_assertions)]
        self.debug_applied_impulses.clear();

        self.entities_buf.clear();
        new_entity_stream(ecs_world)
            .require::<Collider>()
            .require::<C_Spatial2D>()
            .build()
            .collect(ecs_world, &mut self.entities_buf);

        {
            trace!("collision_system::fill_quadtree", _tracer);
            let mut map_collider = unsafe { ecs_world.get_components_map_unsafe::<Collider>() };
            let map_spatial = unsafe { ecs_world.get_components_map_unsafe::<C_Spatial2D>() };

            for &entity in &self.entities_buf {
                let collider = {
                    let collider = unsafe { map_collider.get_component_mut(entity) }.unwrap();
                    collider.colliding = false;
                    *collider
                };

                let transform = &unsafe { map_spatial.get_component(entity) }
                    .unwrap()
                    .global_transform;

                self.quadtree.add(
                    entity,
                    &collider,
                    transform,
                    ecs_world,
                    clone_tracer!(_tracer),
                );
            }
        }

        // Step 2: do collision detection

        {
            trace!("collision_detection_and_solving", _tracer);

            let n_collisions_total = Arc::new(AtomicUsize::new(0));
            let n_entities = self.entities_buf.len();
            self.collided_entities.lock().unwrap().clear();

            {
                trace!("collision_detection", _tracer);

                thread::scope(|s| {
                    let n_threads = num_cpus::get();
                    for ent_chunk in self.entities_buf.chunks(n_entities / n_threads + 1) {
                        let _tracer = clone_tracer!(_tracer);
                        let quadtree = &self.quadtree;
                        let n_collisions_total = n_collisions_total.clone();
                        let collided_entities = self.collided_entities.clone();
                        let ecs_world = ecs_world as &Ecs_World;
                        s.spawn(move |_| {
                            let map_collider = ecs_world.get_components_map::<Collider>();
                            let map_spatial = ecs_world.get_components_map::<C_Spatial2D>();
                            let mut neighbours = vec![];
                            for &entity in ent_chunk {
                                let collider = map_collider.get_component(entity).unwrap();
                                let spatial = map_spatial.get_component(entity).unwrap();
                                let transform = &spatial.global_transform;
                                let velocity = spatial.velocity;

                                neighbours.clear();
                                quadtree.get_neighbours(
                                    collider,
                                    transform,
                                    &mut neighbours,
                                    clone_tracer!(_tracer),
                                );
                                if !neighbours.is_empty() {
                                    check_collision_with_neighbours(
                                        entity,
                                        collider,
                                        transform,
                                        velocity,
                                        &neighbours,
                                        &map_collider,
                                        &map_spatial,
                                        collided_entities.clone(),
                                        n_collisions_total.clone(),
                                    );
                                }
                            }
                        });
                    }
                })
                .unwrap();
            }

            {
                trace!("collision_solving", _tracer);

                // @Audit: is this safe to do?
                let mut map_collider = unsafe { ecs_world.get_components_map_unsafe::<Collider>() };
                let mut map_spatial =
                    unsafe { ecs_world.get_components_map_unsafe::<C_Spatial2D>() };

                if let Ok(cld) = self.collided_entities.lock() {
                    for (&entity, info) in cld.iter() {
                        if info.my_velocity.magnitude2().abs() < 0.0001 {
                            continue;
                        }
                        {
                            let collider =
                                unsafe { map_collider.get_component_mut(entity) }.unwrap();
                            if collider.colliding {
                                continue; // already processed
                            }
                            collider.colliding = true;
                        }
                        {
                            let oth_collider =
                                unsafe { map_collider.get_component_mut(info.other) }.unwrap();
                            oth_collider.colliding = true;
                        }

                        // @Incomplete: solve the collision
                        let spatial = unsafe { map_spatial.get_component_mut(entity) }.unwrap();
                        let delta_pos = info.oth_pos - info.my_pos;

                        // Reset velocity
                        spatial.velocity = Vec2f::default();
                        // Move out of the collision
                        spatial.global_transform.translate_v(
                            -delta_pos.normalized_or_zero() * (0.01 + info.penetration_dist),
                        );

                        #[cfg(debug_assertions)]
                        self.debug_applied_impulses.push(Debug_Applied_Impulse {
                            center: spatial.global_transform.position(),
                            impulse: -delta_pos.normalized_or_zero()
                                * (0.01 + info.penetration_dist),
                        });
                    }
                }
            }

            //println!(
            //"tot collisions: {}, average: {}",
            //n_collisions_total.load(std::sync::atomic::Ordering::SeqCst),
            //n_collisions_total.load(std::sync::atomic::Ordering::SeqCst) / n_entities
            //);
            ()
        }
    }
}

/// Given the Entity `entity` with its `collider` and `transform`, and given an array of `neighbours`,
/// computes collisions between that entity and its neighbours, adding each colliding entity to the
/// `collided_entities` array and incrementing the total number of collisions.
#[allow(clippy::too_many_arguments)]
fn check_collision_with_neighbours(
    entity: Entity,
    collider: &Collider,
    transform: &Transform2D,
    velocity: Vec2f,
    neighbours: &[Entity],
    map_collider: &Components_Map_Safe<Collider>,
    map_spatial: &Components_Map_Safe<C_Spatial2D>,
    collided_entities: Arc<Mutex<HashMap<Entity, Collision_Info>>>,
    n_collisions_total: Arc<AtomicUsize>,
) {
    let pos = transform.position() + collider.offset;
    let scale = transform.scale();

    for &neighbour in neighbours {
        if neighbour == entity {
            continue;
        }

        let oth_cld = map_collider.get_component(neighbour).unwrap();
        let oth_spatial = map_spatial.get_component(neighbour).unwrap();
        let oth_transf = &oth_spatial.global_transform;
        let oth_velocity = oth_spatial.velocity;
        let oth_pos = oth_transf.position() + oth_cld.offset;
        let oth_scale = oth_transf.scale();

        match collider.shape {
            Collider_Shape::Rect { width, height } => match oth_cld.shape {
                Collider_Shape::Rect {
                    width: oth_width,
                    height: oth_height,
                } => {
                    let me = Rect::new(pos.x, pos.y, width * scale.x, height * scale.y);
                    let him = Rect::new(
                        oth_pos.x,
                        oth_pos.y,
                        oth_width * oth_scale.x,
                        oth_height * oth_scale.y,
                    );
                    if let Some(intersection) = rect::rects_intersection(&me, &him) {
                        if let Ok(mut cld) = collided_entities.lock() {
                            cld.insert(
                                entity,
                                Collision_Info {
                                    my_pos: pos,
                                    other: neighbour,
                                    oth_pos,
                                    my_velocity: velocity,
                                    oth_velocity,
                                    penetration_dist: 0., // @Incomplete
                                },
                            );
                            cld.insert(
                                neighbour,
                                Collision_Info {
                                    my_pos: oth_pos,
                                    other: entity,
                                    oth_pos: pos,
                                    my_velocity: oth_velocity,
                                    oth_velocity: velocity,
                                    penetration_dist: 0., // @Incomplete
                                },
                            );
                        }
                        n_collisions_total
                            .fetch_add(neighbours.len(), std::sync::atomic::Ordering::Relaxed);
                    }
                }
                Collider_Shape::Circle { .. } => {
                    // @Incomplete
                    eprintln!("[ TODO ] rect-circle collisions are not implemented yet.");
                }
            },
            Collider_Shape::Circle { radius } => match oth_cld.shape {
                Collider_Shape::Circle { radius: oth_radius } => {
                    let me = shapes::Circle {
                        center: Vec2f::new(pos.x, pos.y),
                        // Note: we assume uniform scale
                        radius: radius * scale.x,
                    };
                    let him = shapes::Circle {
                        center: Vec2f::new(oth_pos.x, oth_pos.y),
                        // Note: we assume uniform scale
                        radius: oth_radius * oth_scale.x,
                    };
                    let penetration_dist = me.penetration_distance(&him);
                    if penetration_dist > 0. {
                        if let Ok(mut cld) = collided_entities.lock() {
                            cld.insert(
                                entity,
                                Collision_Info {
                                    my_pos: pos,
                                    other: neighbour,
                                    oth_pos,
                                    my_velocity: velocity,
                                    oth_velocity,
                                    penetration_dist,
                                },
                            );
                            cld.insert(
                                neighbour,
                                Collision_Info {
                                    my_pos: oth_pos,
                                    other: entity,
                                    oth_pos: pos,
                                    my_velocity: oth_velocity,
                                    oth_velocity: velocity,
                                    penetration_dist,
                                },
                            );
                        }
                        n_collisions_total
                            .fetch_add(neighbours.len(), std::sync::atomic::Ordering::Relaxed);
                    }
                }
                Collider_Shape::Rect { .. } => {
                    // @Incomplete
                    eprintln!("[ TODO ] rect-circle collisions are not implemented yet.");
                }
            },
        }
    }
}
