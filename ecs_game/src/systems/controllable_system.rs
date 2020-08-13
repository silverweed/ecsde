use crate::input_utils::{get_movement_from_input, Input_Config};
use inle_cfg::{self, Cfg_Var};
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::Ecs_World;
use inle_input::axes::Virtual_Axes;
use inle_input::input_state::Game_Action;
use inle_math::vector::Vec2f;
use std::time::Duration;

#[derive(Copy, Clone, Debug)]
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
    ecs_world: &mut Ecs_World,
    input_cfg: Input_Config,
    cfg: &inle_cfg::Config,
) {
    let movement = get_movement_from_input(axes, input_cfg, cfg);
    let dt_secs = dt.as_secs_f32();

    foreach_entity!(ecs_world, +C_Controllable, +C_Spatial2D, |entity| {
        let ctrl = ecs_world
            .get_component_mut::<C_Controllable>(entity)
            .unwrap();
        let speed = ctrl.speed.read(cfg);
        let velocity = movement * speed;
        let v = velocity * dt_secs;
        ctrl.translation_this_frame = v;

        let spatial = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        spatial.velocity = velocity;
    });
}
