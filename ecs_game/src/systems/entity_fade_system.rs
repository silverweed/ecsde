use ecs_engine::collisions::collider::C_Collider;
use ecs_engine::ecs::components::gfx::{C_Multi_Renderable, C_Renderable};
use smallvec::SmallVec;
use ecs_engine::collisions::phys_world::{Collider_Handle, Physics_World};
use ecs_engine::ecs::ecs_world::{Ecs_World, Entity};
use ecs_engine::core::app::Engine_State;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Fade_On_Contact {
    pub trigger_handle: Collider_Handle,
    pub fade_duration: Duration,
}

#[derive(Copy, Clone)]
enum Fade_Type {
    Out,
    In
}

#[derive(Default)]
pub struct Entity_Fade_System {
    entities: HashMap<Entity, (Fade_Type, Duration)>,
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
                            match  self.entities.get(&entity) {
                                 Some((Fade_Type::In, dur)) => {
                                     let dur = *dur;
                                     self.entities.insert(entity, (Fade_Type::Out, dur));
                                 },
                                 None => {
                                     self.entities.insert(entity,  (Fade_Type::Out, Duration::default()));
                                    },
                                 _ => {}
                            }
                        } else {
                            // Start fade in
                            match  self.entities.get(&entity) {
                                 Some((Fade_Type::Out, dur)) => {
                                     let dur = *dur;
                                     self.entities.insert(entity,  (Fade_Type::In, dur));
                                 },
                                 None => {self.entities.insert(entity, (Fade_Type::In, Duration::default()));},
                                 _ => {}
                            }
                        }
                    }
                    break;
                }
            }
        });

        for (&entity, &(fade_type, dur)) in &self.entities {
            let mut renderables: SmallVec<[&mut C_Renderable; C_Multi_Renderable::MAX_RENDERABLES]> = SmallVec::default();

            if let Some(rend) = world.get_component_mut::<C_Renderable>(entity) {
                renderables.push(rend);
            } else {
                let multi_rend = world.get_component_mut::<C_Multi_Renderable>(entity).unwrap();
                for rend in &mut multi_rend.renderables {
                    renderables.push(rend);
                }
            }
        }
    }
}
