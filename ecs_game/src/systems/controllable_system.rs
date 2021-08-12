use crate::input_utils::{get_movement_from_input, Input_Config};
use crate::systems::ground_detection_system::C_Ground_Detection;
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
    pub max_jumps: Cfg_Var<i32>,

    pub n_jumps_done: u32,
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

    foreach_entity!(ecs_world, +C_Controllable, +C_Spatial2D, +C_Ground_Detection, |entity| {
        let ctrl = ecs_world
            .get_component::<C_Controllable>(entity)
            .unwrap();
        let acceleration = ctrl.acceleration.read(cfg);
        let jump_impulse = ctrl.jump_impulse.read(cfg);
        let dampening = ctrl.dampening.read(cfg);
        let horiz_max_speed = ctrl.horiz_max_speed.read(cfg);

        let ground_detect = ecs_world.get_component::<C_Ground_Detection>(entity).unwrap();
        let reset_jumps = ground_detect.just_touched_ground;

        let can_jump = (if reset_jumps { 0 } else { ctrl.n_jumps_done }) < ctrl.max_jumps.read(cfg) as u32;

        let spatial = ecs_world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        let velocity = &mut spatial.velocity;

        velocity.x += movement * acceleration * dt_secs;

        let mut jumped = false;
        if can_jump && actions.contains(&(sid!("jump"), Action_Kind::Pressed)) {
            velocity.y -= jump_impulse;
            jumped = true;
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
        if reset_jumps {
            ctrl.n_jumps_done = 0;
        }
        ctrl.n_jumps_done += jumped as u32;
    });
}
