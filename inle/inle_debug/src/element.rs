use inle_alloc::temp;
use inle_cfg::Config;
use inle_gfx::render_window::Render_Window_Handle;
use inle_input::input_state::Input_State;
use inle_resources::gfx::Gfx_Resources;
use std::time::Duration;

pub struct Update_Args<'a> {
    pub dt: &'a Duration,
    pub window: &'a Render_Window_Handle,
    pub input_state: &'a Input_State,
    pub config: &'a Config,
}

pub struct Draw_Args<'a, 'r> {
    pub window: &'a mut Render_Window_Handle,
    pub gres: &'a mut Gfx_Resources<'r>,
    pub input_state: &'a Input_State,
    pub frame_alloc: &'a mut temp::Temp_Allocator,
    pub config: &'a Config,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Update_Res {
    Stay_Enabled,
    Disable_Self,
}

pub trait Debug_Element {
    fn update(&mut self, _args: Update_Args) -> Update_Res {
        Update_Res::Stay_Enabled
    }

    fn draw(&self, args: Draw_Args);
}
