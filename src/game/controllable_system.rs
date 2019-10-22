use crate::cfg::{from_cfg, Cfg_Var};
use crate::core::common::vector::Vec2f;
use crate::core::time;
use crate::ecs::entity_manager::Entity_Manager;
use crate::input::axes::Virtual_Axes;
use crate::input::input_system::Game_Action;
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

pub fn update(
    dt: &Duration,
    _actions: &[Game_Action],
    axes: &Virtual_Axes,
    em: &mut Entity_Manager,
) {
    let movement = super::gameplay_system::get_normalized_movement_from_input(axes);
    let dt_secs = time::to_secs_frac(&dt);
    let controllables = em.get_components_mut::<C_Controllable>();

    for mut ctrl in controllables {
        let speed = from_cfg(ctrl.speed);
        let velocity = movement * speed;
        let v = velocity * dt_secs;
        ctrl.translation_this_frame = v;
    }
}
