use crate::core::common::colors::Color;
use crate::core::common::rect::Rectf;
use crate::core::common::transform::Transform2D;
use crate::gfx::render;
use crate::gfx::window::Window_Handle;

pub struct Debug_Painter {
    rects: Vec<(Rectf, Transform2D, Color)>,
}

impl Debug_Painter {
    pub fn new() -> Self {
        Debug_Painter { rects: vec![] }
    }

    pub fn add_rect(&mut self, rect: Rectf, transform: &Transform2D, color: Color) {
        self.rects.push((rect, *transform, color));
    }

    pub fn clear(&mut self) {
        self.rects.clear();
    }

    pub fn draw(&self, window: &mut Window_Handle, camera: &Transform2D) {
        for (rect, transform, col) in &self.rects {
            render::fill_color_rect_ws(window, *col, *rect, transform, camera);
        }
    }
}
