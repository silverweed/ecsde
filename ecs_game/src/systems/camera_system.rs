use inle_cfg::{Cfg_Var, Config};
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::{Ecs_World, Entity};
use inle_gfx::components::C_Camera2D;
use inle_gfx::render_window::Render_Window_Handle;
use inle_math::vector::Vec2f;
use std::time::Duration;

#[derive(Copy, Clone, Debug)]
pub enum Camera_Follow_Target {
    None,
    Position(Vec2f),
    Entity(Entity),
}

impl Default for Camera_Follow_Target {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Copy, Clone, Default)]
pub struct C_Camera_Follow {
    pub target: Camera_Follow_Target,
    pub lerp_factor: Cfg_Var<f32>,
}

pub fn update(dt: &Duration, world: &mut Ecs_World, window: &Render_Window_Handle, cfg: &Config) {
    let (win_w, win_h) = inle_win::window::get_window_target_size(window);

    foreach_entity!(world, +C_Camera_Follow, +C_Camera2D, |entity| {
        let cam_follow = world.get_component::<C_Camera_Follow>(entity).unwrap();
        let target = cam_follow.target;
        let lerp_factor = cam_follow.lerp_factor.read(cfg);

        let camera = world.get_component::<C_Camera2D>(entity).unwrap();

        let target_pos = match target {
            Camera_Follow_Target::None => { return; },
            Camera_Follow_Target::Position(pos) => pos,
            Camera_Follow_Target::Entity(entity) => {
                let spatial = world.get_component::<C_Spatial2D>(entity).expect("Followed entity has no C_Spatial2D!");
                spatial.transform.position() / camera.transform.scale()
            },
        };

        let camera = world.get_component_mut::<C_Camera2D>(entity).unwrap();
        let cam_pos = camera.transform.position();
        let diff = target_pos - cam_pos;
        camera.transform.translate_v(lerp_factor * diff * dt.as_secs_f32());
    });
}
