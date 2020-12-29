use inle_alloc::temp;
use inle_gfx::render_window::Render_Window_Handle;
use inle_input::input_state::Input_State;
use inle_resources::gfx::Gfx_Resources;
use std::time::Duration;

pub trait Debug_Element {
    fn update(
        &mut self,
        _dt: &Duration,
        _window: &Render_Window_Handle,
        _input_state: &Input_State,
    ) {
    }
    fn draw(
        &self,
        window: &mut Render_Window_Handle,
        gres: &mut Gfx_Resources,
        input_state: &Input_State,
        frame_alloc: &mut temp::Temp_Allocator,
    );
}
