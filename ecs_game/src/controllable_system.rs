use crate::input_utils::{get_movement_from_input, Input_Config};
use ecs_engine::cfg::{self, Cfg_Var};
use ecs_engine::core::common::vector::Vec2f;
use ecs_engine::core::time;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::ecs_world::Ecs_World;
use ecs_engine::ecs::entity_stream::new_entity_stream;
use ecs_engine::input::axes::Virtual_Axes;
use ecs_engine::input::input_system::Game_Action;
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
    cfg: &cfg::Config,
) {
    let movement = get_movement_from_input(axes, input_cfg, cfg);
    let dt_secs = time::to_secs_frac(&dt);

    let mut entity_stream = new_entity_stream(ecs_world)
        .require::<C_Controllable>()
        .require::<C_Spatial2D>()
        .build();
    loop {
        let entity = entity_stream.next(ecs_world);
        if entity.is_none() {
            break;
        }
        let entity = entity.unwrap();
        let ctrl = ecs_world
            .get_component_mut::<C_Controllable>(entity)
            .unwrap();
        let speed = ctrl.speed.read(cfg);
        let velocity = movement * speed;
        let v = velocity * dt_secs;
        ctrl.translation_this_frame = v;

        let spatial = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        spatial.velocity = velocity;
    }
}
