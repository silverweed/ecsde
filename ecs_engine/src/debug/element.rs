use crate::alloc::temp;
use crate::input::input_system::Input_State;
use crate::gfx::window::Window_Handle;
use crate::resources::gfx::Gfx_Resources;
use std::time::Duration;

pub trait Debug_Element {
    fn update(&mut self, _dt: &Duration, _window: &Window_Handle, _input_state: &Input_State) {}
    fn draw(
        &self,
        window: &mut Window_Handle,
        gres: &mut Gfx_Resources,
        frame_alloc: &mut temp::Temp_Allocator,
    );
}
