use crate::input_utils::Input_Config;
use inle_app::app::Engine_State;
use inle_ecs::ecs_query_new::Ecs_Query;
use inle_ecs::ecs_world::{Ecs_World, Entity};
use inle_gfx::render_window::Render_Window_Handle;
use inle_physics::phys_world::Physics_World;
use std::time::Duration;

pub struct Update_Args<'a, 'e> {
    pub dt: Duration,
    pub ecs_world: &'a mut Ecs_World,
    pub phys_world: &'a mut Physics_World,
    pub engine_state: &'a mut Engine_State<'e>,
    pub input_cfg: &'a Input_Config,
}

pub struct Realtime_Update_Args<'a, 'e> {
    pub dt: Duration,
    pub window: &'a Render_Window_Handle,
    pub ecs_world: &'a mut Ecs_World,
    pub engine_state: &'a mut Engine_State<'e>,
    pub input_cfg: &'a Input_Config,

    // @Cleanup: ugly
    pub cameras: &'a [Entity],
    pub active_camera: usize,
}

pub trait Game_System {
    fn get_queries_mut(&mut self) -> Vec<&mut Ecs_Query>;

    fn update(&self, _args: &mut Update_Args) {}

    fn realtime_update(&self, _args: &mut Realtime_Update_Args) {}
}
