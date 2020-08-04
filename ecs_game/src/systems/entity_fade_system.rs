use ecs_engine::collisions::collider::C_Collider;
use ecs_engine::collisions::phys_world::{Collider_Handle, Physics_World};
use ecs_engine::common::math::lerp;
use ecs_engine::core::app::Engine_State;
use ecs_engine::core::time;
use ecs_engine::ecs::components::gfx::{C_Multi_Renderable, C_Renderable};
use ecs_engine::ecs::ecs_world::{Ecs_World, Entity};
use smallvec::SmallVec;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Fade_On_Contact {
    pub trigger_handle: Collider_Handle,
    pub fade_duration: Duration,
    pub min_alpha: u8,
}

#[derive(Copy, Clone, Debug)]
enum Fade_Type {
    Out,
    In,
}

#[derive(Copy, Clone, Debug)]
struct Entity_Fade_Data {
    pub fade_type: Fade_Type,
    pub t: Duration,
    pub duration: Duration,
    pub min_alpha: u8,
}

#[derive(Default)]
pub struct Entity_Fade_System {
    entities: HashMap<Entity, Entity_Fade_Data>,
}

impl Entity_Fade_System {
    pub fn update(&mut self, world: &mut Ecs_World, phys_world: &Physics_World, dt: &Duration) {
        foreach_entity!(world, +C_Fade_On_Contact, +C_Collider, |entity| {
            let fade_on_contact = world.get_component::<C_Fade_On_Contact>(entity).unwrap();
            let collider = world.get_component::<C_Collider>(entity).unwrap();
            for (cld, handle) in phys_world.get_all_colliders_with_handles(collider.handle) {
                if handle == fade_on_contact.trigger_handle {
                    if world.has_component::<C_Renderable>(entity) || world.has_component::<C_Multi_Renderable>(entity) {
                        if !cld.colliding_with.is_empty() {
                            // Start fade out
                            match self.entities.get(&entity) {
                                Some(&Entity_Fade_Data {
                                    fade_type: Fade_Type::In,
                                    t,
                                    duration,
                                    min_alpha
                                }) => {
                                    self.entities.insert(entity, Entity_Fade_Data {
                                        fade_type: Fade_Type::Out,
                                        t,
                                        duration,
                                        min_alpha,
                                    });
                                },
                                None => {
                                    self.entities.insert(entity, Entity_Fade_Data {
                                        fade_type: Fade_Type::Out,
                                        t: Duration::default(),
                                        duration: fade_on_contact.fade_duration,
                                        min_alpha: fade_on_contact.min_alpha
                                    });
                                },
                                 _ => {}
                            }
                        } else {
                            // Start fade in
                            match self.entities.get(&entity) {
                                Some(&Entity_Fade_Data {
                                    fade_type: Fade_Type::Out,
                                    t,
                                    duration,
                                    min_alpha
                                }) => {
                                    self.entities.insert(entity, Entity_Fade_Data {
                                         fade_type: Fade_Type::In,
                                         t,
                                         duration,
                                         min_alpha
                                    });
                                },
                                None => {
                                    self.entities.insert(entity, Entity_Fade_Data {
                                        fade_type: Fade_Type::In,
                                        t: Duration::default(),
                                        duration: fade_on_contact.fade_duration,
                                        min_alpha: fade_on_contact.min_alpha
                                    });
                                },
                                _ => {}
                            }
                        }
                    }
                    break;
                }
            }
        });

        for (&entity, data) in &mut self.entities {
            let mut renderables: SmallVec<
                [&mut C_Renderable; C_Multi_Renderable::MAX_RENDERABLES],
            > = SmallVec::default();

            let has_renderable = world.has_component::<C_Renderable>(entity);
            if has_renderable {
                let rend = world.get_component_mut::<C_Renderable>(entity).unwrap();
                renderables.push(rend);
            } else {
                let multi_rend = world
                    .get_component_mut::<C_Multi_Renderable>(entity)
                    .unwrap();
                for rend in &mut multi_rend.renderables {
                    renderables.push(rend);
                }
            }

            for rend in renderables {
                rend.modulate.a = lerp(
                    data.min_alpha as f32,
                    255.0,
                    time::duration_ratio(&data.t, &data.duration),
                ) as u8;
            }

            let fade_type = data.fade_type;
            match fade_type {
                Fade_Type::In => data.t = (data.t + *dt).min(data.duration),
                Fade_Type::Out => {
                    if data.t > *dt {
                        data.t -= *dt
                    } else {
                        data.t = Duration::default()
                    }
                }
            }
        }
    }
}
