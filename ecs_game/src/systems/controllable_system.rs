use super::interface::{Game_System, Update_Args};
use crate::input_utils::get_movement_from_input;
use crate::systems::ground_detection_system::C_Ground_Detection;
use inle_cfg::config::Config;
use inle_cfg::{self, Cfg_Var};
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_query_new::Ecs_Query;
use inle_input::input_state::Action_Kind;
use inle_math::math::clamp;
use inle_math::vector::Vec2f;

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Controllable {
    pub translation_this_frame: Vec2f,
    pub speed: Cfg_Var<f32>, // @Cleanup: this is currently used by the camera!
    pub acceleration: Cfg_Var<f32>,
    pub jump_impulse: Cfg_Var<f32>,
    pub dampening: Cfg_Var<f32>,
    pub horiz_max_speed: Cfg_Var<f32>,
    pub vert_max_speed: Cfg_Var<f32>,
    pub max_jumps: Cfg_Var<i32>,

    pub n_jumps_done: u32,
}

pub struct Controllable_System {
    query: Ecs_Query,
    camera_on_player: Cfg_Var<bool>,
}

impl Controllable_System {
    pub fn new(cfg: &Config) -> Self {
        Self {
            query: Ecs_Query::new()
                .read::<C_Ground_Detection>()
                .write::<C_Controllable>()
                .write::<C_Spatial2D>(),
            camera_on_player: Cfg_Var::new("game/camera/on_player", cfg),
        }
    }
}

impl Game_System for Controllable_System {
    fn get_queries_mut(&mut self) -> Vec<&mut Ecs_Query> {
        vec![&mut self.query]
    }

    fn update(&self, args: &mut Update_Args) {
        let Update_Args {
            dt,
            engine_state,
            ecs_world,
            input_cfg,
            ..
        } = args;

        let axes = &engine_state.input_state.processed.virtual_axes;
        let actions = &engine_state.input_state.processed.game_actions;
        let cfg = &engine_state.config;

        if !self.camera_on_player.read(cfg) {
            return;
        }

        let movement = get_movement_from_input(axes, **input_cfg, cfg).x;
        let dt_secs = dt.as_secs_f32();

        foreach_entity!(ecs_world,
            read: C_Ground_Detection;
            write: C_Controllable, C_Spatial2D;
            |_e, (ground_detect,): (&C_Ground_Detection,), (ctrl, spatial): (&mut C_Controllable, &mut C_Spatial2D)| {
            let acceleration = ctrl.acceleration.read(cfg);
            let jump_impulse = ctrl.jump_impulse.read(cfg);
            let dampening = ctrl.dampening.read(cfg);
            let horiz_max_speed = ctrl.horiz_max_speed.read(cfg);
            let vert_max_speed = ctrl.vert_max_speed.read(cfg);

            let reset_jumps = ground_detect.just_touched_ground;

            let can_jump = (if reset_jumps { 0 } else { ctrl.n_jumps_done }) < ctrl.max_jumps.read(cfg) as u32;

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
            velocity.x = clamp(velocity.x, -horiz_max_speed, horiz_max_speed);
            velocity.y = clamp(velocity.y, -vert_max_speed, vert_max_speed);

            let v = *velocity * dt_secs;
            ctrl.translation_this_frame = v;
            if reset_jumps {
                ctrl.n_jumps_done = 0;
            }
            ctrl.n_jumps_done += jumped as u32;
        });
    }
}
