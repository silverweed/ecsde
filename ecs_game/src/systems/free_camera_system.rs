use super::interface::{Game_System, Realtime_Update_Args};
use crate::input_utils::get_movement_from_input;
use crate::systems::controllable_system::C_Controllable;
use inle_cfg::{Cfg_Var, Config};
use inle_ecs::ecs_query_new::Ecs_Query;
use inle_gfx::components::C_Camera2D;
use inle_input::input_state::Action_Kind;

pub struct Free_Camera_System {
    camera_on_player: Cfg_Var<bool>,
}

impl Free_Camera_System {
    pub fn new(cfg: &Config) -> Self {
        Self {
            camera_on_player: Cfg_Var::new("game/camera/on_player", cfg),
        }
    }
}

impl Game_System for Free_Camera_System {
    fn get_queries_mut(&mut self) -> Vec<&mut Ecs_Query> {
        vec![]
    }

    fn realtime_update(&self, args: &mut Realtime_Update_Args) {
        let Realtime_Update_Args {
            dt,
            engine_state,
            input_cfg,
            ecs_world,
            cameras,
            active_camera,
            window,
            ..
        } = args;

        let input_state = &engine_state.input_state;
        let cfg = &engine_state.config;

        if !self.camera_on_player.read(cfg) {
            return;
        }

        let movement =
            get_movement_from_input(&input_state.processed.virtual_axes, **input_cfg, cfg);

        let cam_translation = {
            let camera_ctrl =
                ecs_world.get_component_mut::<C_Controllable>(cameras[*active_camera]);
            if camera_ctrl.is_none() {
                return;
            }

            let dt_secs = dt.as_secs_f32();
            let mut camera_ctrl = camera_ctrl.unwrap();
            let speed = camera_ctrl.speed.read(cfg);
            let velocity = movement * speed;
            let cam_translation = velocity * dt_secs;
            camera_ctrl.translation_this_frame = cam_translation;
            cam_translation
        };

        let mut camera = ecs_world
            .get_component_mut::<C_Camera2D>(cameras[*active_camera])
            .unwrap();

        let sx = camera.transform.scale().x;
        let mut cam_translation = cam_translation * sx;

        let mut add_scale = v2!(0., 0.);
        const BASE_CAM_DELTA_ZOOM_PER_SCROLL: f32 = 0.2;
        let base_delta_zoom_per_scroll =
            Cfg_Var::<f32>::new("game/camera/free/base_delta_zoom_per_scroll", cfg).read(cfg);

        for action in &input_state.processed.game_actions {
            match action {
                (name, Action_Kind::Pressed) if *name == sid!("camera_zoom_up") => {
                    add_scale.x -= base_delta_zoom_per_scroll * sx;
                    add_scale.y = add_scale.x;
                }
                (name, Action_Kind::Pressed) if *name == sid!("camera_zoom_down") => {
                    add_scale.x += base_delta_zoom_per_scroll * sx;
                    add_scale.y = add_scale.x;
                }
                _ => (),
            }
        }

        if add_scale.magnitude2() > 0. {
            // Preserve mouse world position
            let cur_mouse_wpos = inle_gfx::render_window::mouse_pos_in_world(
                window,
                &input_state.raw.mouse_state,
                &camera.transform,
            );

            camera.transform.add_scale_v(add_scale);
            let mut new_scale = camera.transform.scale();
            new_scale.x = new_scale.x.max(0.001);
            new_scale.y = new_scale.y.max(0.001);
            camera.transform.set_scale_v(new_scale);

            let new_mouse_wpos = inle_gfx::render_window::mouse_pos_in_world(
                window,
                &input_state.raw.mouse_state,
                &camera.transform,
            );

            cam_translation += cur_mouse_wpos - new_mouse_wpos;
        }

        camera.transform.translate_v(cam_translation);
    }
}
