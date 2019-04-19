use crate::core::common::transform::C_Transform2D;
use crate::core::input;
use crate::core::time;
use crate::ecs::components::C_Controllable;
use crate::ecs::entity_manager::Entity_Manager;
use std::time::Duration;

pub fn update(dt: &Duration, actions: &input::Action_List, em: &mut Entity_Manager) {
    let movement = input::get_movement_from_input(actions);
    let dt_secs = time::to_secs_frac(&dt);
    let controllables = em.get_component_tuple_first_mut::<C_Transform2D, C_Controllable>();

    for (transf, ctrl) in controllables {
        let speed = ctrl.speed;
        let velocity = movement * speed;
        transf.translate_v(velocity * dt_secs);
    }
}
