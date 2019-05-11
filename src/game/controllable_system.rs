use crate::core::common::transform::C_Transform2D;
use crate::core::input;
use crate::core::time;
use crate::ecs::entity_manager::Entity_Manager;
use std::time::Duration;
use typename::TypeName;

#[derive(Copy, Clone, Default, Debug, TypeName)]
pub struct C_Controllable {
    pub speed: f32,
}

pub fn update(dt: &Duration, actions: &input::Action_List, em: &mut Entity_Manager) {
    let movement = input::get_normalized_movement_from_input(actions);
    let dt_secs = time::to_secs_frac(&dt);
    let controllables = em.get_component_tuple_mut::<C_Transform2D, C_Controllable>();

    for (transf, ctrl) in controllables {
        let speed = ctrl.borrow().speed;
        let velocity = movement * speed;
        let mut transf = transf.borrow_mut();
        transf.translate_v(velocity * dt_secs);
    }
}
