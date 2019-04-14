use crate::core::common::transform::C_Transform2D;
use crate::core::input;
use crate::core::time;
use crate::ecs::components::C_Controllable;
use crate::ecs::entity_manager::{Entity, Entity_Manager};
use std::time::Duration;
use std::vec::Vec;

pub fn update(
    dt: &Duration,
    actions: &input::Action_List,
    entities: &[Entity],
    em: &mut Entity_Manager,
) {
    let movement = input::get_movement_from_input(actions);
    let dt_secs = time::to_secs_frac(&dt);

    // @Cleanup: we'd like to have the Entity_Manager do this for us, so we don't have to pass the
    // slice of Entities to this system as well.
    let controllables: Vec<&Entity> = entities
        .iter()
        .filter(|&&e| em.has_component::<C_Transform2D>(e) && em.has_component::<C_Controllable>(e))
        .collect();

    for &ctrl in controllables {
        let speed = em.get_component::<C_Controllable>(ctrl).unwrap().speed;
        let velocity = movement * speed;
        let tr = em.get_component_mut::<C_Transform2D>(ctrl).unwrap();
        tr.translate_v(velocity * dt_secs);
    }
}
