use crate::ecs::entity_manager::Entity_Manager;
use ecs_engine::cfg::{self, Cfg_Var};
use ecs_engine::core::common::vector::Vec2f;
use ecs_engine::core::time;
use ecs_engine::input::axes::Virtual_Axes;
use ecs_engine::input::input_system::Game_Action;
use std::time::Duration;

#[derive(Clone, Debug)]
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
    cfg: &cfg::Config,
) {
    let movement = super::gameplay_system::get_normalized_movement_from_input(axes);
    let dt_secs = time::to_secs_frac(&dt);
    let controllables = em.get_components_mut::<C_Controllable>();

    for mut ctrl in controllables {
        let speed = ctrl.speed.read(cfg);
        let velocity = movement * speed;
        let v = velocity * dt_secs;
        ctrl.translation_this_frame = v;
    }
}
