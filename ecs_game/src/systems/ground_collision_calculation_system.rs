use crate::directions;
use crate::load::load_system::C_Ground;
use ecs_engine::collisions::collider::{Collider, Collision_Shape};
use ecs_engine::common::vector::Vec2i;
use ecs_engine::core::app::Engine_State;
use ecs_engine::ecs::components::gfx::C_Renderable;
use ecs_engine::ecs::ecs_world::{Ecs_World, Entity, Evt_Entity_Destroyed};
use ecs_engine::events::evt_register::{with_cb_data, wrap_cb_data, Event_Callback_Data};

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

    pub fn update(&mut self, world: &mut Ecs_World) {
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
                            if !world.has_component::<Collider>(e) {
                                world.add_component(
                                    e,
                                    Collider {
                                        shape: Collision_Shape::Rect {
                                            width: width as f32,
                                            height: height as f32,
                                        },
                                        is_static: true,
                                        ..Default::default()
                                    },
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
