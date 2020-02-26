use crate::common::colors;
use crate::common::rect::Rect;
use crate::common::vector::{Vec2f, Vec2u};
use crate::gfx::render;
use crate::gfx::window::Window_Handle;
use crate::resources::gfx::Gfx_Resources;
use std::ops::Range;

#[derive(Default)]
pub struct Debug_Graph_View {
    pub graph: Debug_Graph,
    pub pos: Vec2u,
    pub size: Vec2u,
}

pub struct Debug_Graph {
    pub points: Vec<Vec2f>,
    pub x_range: Range<f32>,
    pub y_range: Range<f32>,
}

impl Debug_Graph_View {
    pub fn draw(&self, window: &mut Window_Handle, gres: &mut Gfx_Resources) {
        // Draw background
        let Vec2u { x, y } = self.pos;
        let Vec2u { x: w, y: h } = self.size;
        render::fill_color_rect(window, colors::rgba(0, 0, 0, 150), Rect::new(x, y, w, h));
    }
}

impl Default for Debug_Graph {
    fn default() -> Self {
        Self {
            points: vec![],
            x_range: 0.0..0.0,
            y_range: 0.0..0.0,
        }
    }
}

impl Debug_Graph {
    pub fn with_xy_range(x_range: Range<f32>, y_range: Range<f32>) -> Self {
        Debug_Graph {
            points: vec![],
            x_range,
            y_range,
        }
    }
}
