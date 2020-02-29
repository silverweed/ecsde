use crate::gfx::window::Window_Handle;
use crate::prelude::*;
use crate::resources::gfx::Gfx_Resources;
use std::time::Duration;

pub trait Debug_Element {
    fn update(&mut self, _dt: &Duration) {}
    fn draw(&self, window: &mut Window_Handle, gres: &mut Gfx_Resources, _tracer: Debug_Tracer);
}
