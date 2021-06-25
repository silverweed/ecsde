use super::painter::Debug_Painter;
use inle_gfx::render_window::Render_Window_Handle;

mod buf_alloc_debug;

pub fn draw_backend_specific_debug(window: &Render_Window_Handle, painter: &mut Debug_Painter) {
    buf_alloc_debug::debug_draw_buffer_allocators(&window.gl.buffer_allocators, painter);
}
