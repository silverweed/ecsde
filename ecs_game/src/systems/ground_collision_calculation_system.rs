use crate::directions;
use crate::spatial::World_Chunks;
use ecs_engine::collisions::collider::{C_Collider, Collider, Collision_Shape};
use ecs_engine::collisions::phys_world::{Phys_Data, Physics_World};
use ecs_engine::common::vector::Vec2i;
use ecs_engine::core::app::Engine_State;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::components::gfx::C_Renderable;
use ecs_engine::ecs::ecs_world::{Ecs_World, Entity, Evt_Entity_Destroyed};
use ecs_engine::events::evt_register::{with_cb_data, wrap_cb_data, Event_Callback_Data};

#[derive(Copy, Clone, Default)]
pub struct C_Ground {
    pub neighbours: [Entity; 4],
}

pub struct Ground_Collision_Calculation_System {
    entities_to_recalc: Event_Callback_Data,
}

impl Ground_Collision_Calculation_System {
    pub fn new() -> Self {
        Self {
            entities_to_recalc: wrap_cb_data(Vec::<Entity>::new()),
        }
    }

    pub fn init(&mut self, engine_state: &mut Engine_State) {
        engine_state
            .systems
            .evt_register
            .subscribe::<Evt_Entity_Destroyed>(
                Box::new(|entity, to_recalc| {
                    with_cb_data(to_recalc.unwrap(), |to_recalc: &mut Vec<Entity>| {
                        to_recalc.push(entity);
                    });
                }),
                Some(self.entities_to_recalc.clone()),
            );
    }

    pub fn update(
        &mut self,
        world: &mut Ecs_World,
        phys_world: &mut Physics_World,
        chunks: &mut World_Chunks,
    ) {
        with_cb_data(
            &mut self.entities_to_recalc,
            |to_recalc: &mut Vec<Entity>| {
                for &entity in to_recalc.iter() {
                    // We're only interested in rocks.
                    if !world.has_component::<C_Ground>(entity) {
                        continue;
                    }
                    let ground = world.get_component::<C_Ground>(entity).unwrap();
                    let neighs = ground.neighbours;

                    for &i in &directions::square_directions() {
                        let e = neighs[i as usize];
                        if world.is_valid_entity(e) {
                            let renderable = world.get_component::<C_Renderable>(e).unwrap();
                            let Vec2i {
                                x: width,
                                y: height,
                            } = renderable.rect.size();
                            if !world.has_component::<C_Collider>(e) {
                                let shape = Collision_Shape::Rect {
                                    width: width as f32,
                                    height: height as f32,
                                };
                                let cld = Collider {
                                    shape,
                                    is_static: true,
                                    ..Default::default()
                                };
                                // @Incomplete
                                let body_handle = phys_world
                                    .new_physics_body_with_rigidbody(cld, Phys_Data::default());
                                world.add_component(
                                    e,
                                    C_Collider {
                                        handle: body_handle,
                                    },
                                );
                                let pos = world
                                    .get_component::<C_Spatial2D>(e)
                                    .unwrap()
                                    .transform
                                    .position();
                                // @Incomplete :MultipleRigidbodies
                                let body = phys_world.get_physics_body(body_handle).unwrap();
                                chunks.add_collider(
                                    body.rigidbody_colliders[0].0,
                                    pos,
                                    shape.extent(),
                                );
                            }
                        }
                    }
                }
                to_recalc.clear();
            },
        );
    }
}
