use crate::input_utils::{get_movement_from_input, Input_Config};
use inle_cfg::{self, Cfg_Var};
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::Ecs_World;
use inle_input::axes::Virtual_Axes;
use inle_input::input_state::{Action_Kind, Game_Action};
use inle_math::vector::Vec2f;
use std::time::Duration;

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Controllable {
    pub translation_this_frame: Vec2f,
    pub speed: Cfg_Var<f32>, // @Cleanup: this is currently used by the camera!
    pub acceleration: Cfg_Var<f32>,
    pub jump_impulse: Cfg_Var<f32>,
    pub dampening: Cfg_Var<f32>,
    pub horiz_max_speed: Cfg_Var<f32>,
}

pub fn update(
    dt: &Duration,
    actions: &[Game_Action],
    axes: &Virtual_Axes,
    ecs_world: &mut Ecs_World,
    input_cfg: Input_Config,
    cfg: &inle_cfg::Config,
) {
    let movement = get_movement_from_input(axes, input_cfg, cfg).x;
    let dt_secs = dt.as_secs_f32();

    foreach_entity!(ecs_world, +C_Controllable, +C_Spatial2D, |entity| {
        let ctrl = ecs_world
            .get_component::<C_Controllable>(entity)
            .unwrap();
        let acceleration = ctrl.acceleration.read(cfg);
        let jump_impulse = ctrl.jump_impulse.read(cfg);
        let dampening = ctrl.dampening.read(cfg);
        let horiz_max_speed = ctrl.horiz_max_speed.read(cfg);

        let spatial = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        let velocity = &mut spatial.velocity;

        velocity.x += movement * acceleration * dt_secs;
        if actions.contains(&(sid!("jump"), Action_Kind::Pressed)) {
            velocity.y -= jump_impulse;
        }
        let velocity_norm = velocity.normalized_or_zero();
        let speed = velocity.magnitude();
        *velocity -=  velocity_norm * dampening * speed * dt_secs;
        velocity.x = velocity.x.min(horiz_max_speed);

        let v = *velocity * dt_secs;
        let ctrl = ecs_world
            .get_component_mut::<C_Controllable>(entity)
            .unwrap();
        ctrl.translation_this_frame = v;
    });
}
