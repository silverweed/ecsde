use crate::cfg::Cfg_Var;
use crate::core::common::vector::Vec2f;
use crate::core::time;
use crate::ecs::entity_manager::Entity_Manager;
use crate::input::input_system;
use std::time::Duration;
use typename::TypeName;

#[derive(Clone, Debug, TypeName)]
pub struct C_Controllable {
    pub speed: Cfg_Var<f32>,
    pub translation_this_frame: Vec2f,
}

impl Default for C_Controllable {
    fn default() -> C_Controllable {
        C_Controllable {
            speed: Cfg_Var::default(),
            translation_this_frame: Vec2f::new(0.0, 0.0),
        }
    }
}

pub fn update(dt: &Duration, actions: &input_system::Action_List, em: &mut Entity_Manager) {
    let movement = input_system::get_normalized_movement_from_input(actions);
    let dt_secs = time::to_secs_frac(&dt);
    let controllables = em.get_components_mut::<C_Controllable>();

    for mut ctrl in controllables {
        let speed = *ctrl.speed;
        let velocity = movement * speed;
        let v = velocity * dt_secs;
        ctrl.translation_this_frame = v;
    }
}
