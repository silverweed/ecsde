use crate::core::common::color::Norm_Color;
use sdl2::video::Window;

#[inline(always)]
pub fn clear_with_color(c: Norm_Color) {
    unsafe {
        gl::ClearColor(c.r, c.g, c.b, c.a);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}
