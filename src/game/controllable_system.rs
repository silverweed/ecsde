use crate::cfg::Cfg_Var;
use crate::core::input;
use crate::core::time;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::entity_manager::Entity_Manager;
use std::time::Duration;
use typename::TypeName;

#[derive(Clone, Default, Debug, TypeName)]
pub struct C_Controllable {
    pub speed: Cfg_Var<f32>,
}

pub fn update(dt: &Duration, actions: &input::Action_List, em: &mut Entity_Manager) {
    let movement = input::get_normalized_movement_from_input(actions);
    let dt_secs = time::to_secs_frac(&dt);
    let controllables = em.get_component_tuple_mut::<C_Spatial2D, C_Controllable>();

    for (spatial, ctrl) in controllables {
        let speed = *ctrl.borrow().speed;
        let velocity = movement * speed;
        let mut spatial = spatial.borrow_mut();
        let v = velocity * dt_secs;
        spatial.velocity.set(v);
        spatial.transform.translate_v(v);
    }
}
