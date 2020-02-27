use super::element::Debug_Element;
use crate::common::colors;
use crate::common::rect::Rect;
use crate::common::transform::Transform2D;
use crate::common::vector::{Vec2f, Vec2u};
use crate::gfx::render;
use crate::gfx::window::Window_Handle;
use crate::prelude::*;
use crate::resources::gfx::{Font_Handle, Gfx_Resources};
use std::collections::VecDeque;
use std::ops::Range;

#[derive(Default, Clone)]
pub struct Debug_Graph_View_Config {
    pub grid_xstep: Option<f32>,
    pub grid_ystep: Option<f32>,
    pub font: Font_Handle,
    pub label_font_size: u16,
    pub title: Option<String>,
    pub title_font_size: u16,
    pub color: colors::Color,
    // If value is > than this, use this other color
    pub low_threshold: Option<(f32, colors::Color)>,
    // If value is < than this, use this other color
    pub high_threshold: Option<(f32, colors::Color)>,
}

#[derive(Default)]
pub struct Debug_Graph_View {
    pub data: Debug_Graph,
    pub pos: Vec2u,
    pub size: Vec2u,
    pub config: Debug_Graph_View_Config,
}

/// Note: for simplicify, the graph assumes points are added in x-sorted order.
pub struct Debug_Graph {
    pub points: VecDeque<Vec2f>,
    pub x_range: Range<f32>,
    pub y_range: Range<f32>,
}

impl Debug_Element for Debug_Graph_View {
    fn draw(&self, window: &mut Window_Handle, gres: &mut Gfx_Resources, _tracer: Debug_Tracer) {
        trace!("graph::draw", _tracer);

        // Draw background
        let Vec2u { x, y } = self.pos;
        let Vec2u { x: w, y: h } = self.size;
        render::fill_color_rect(window, colors::rgba(0, 0, 0, 200), Rect::new(x, y, w, h));

        let xr = &self.data.x_range;
        let yr = &self.data.y_range;

        // Draw grid
        let font = gres.get_font(self.config.font);
        let font_size = self.config.label_font_size;
        if let Some(xstep) = self.config.grid_xstep {
            let mut x = xr.start;
            let mut iters = 0;
            while x <= xr.end && iters < 100 {
                let pos1 = self.get_coords_for(Vec2f::new(x, yr.start));
                let v1 =
                    render::new_vertex(pos1, colors::rgba(180, 180, 180, 200), Vec2f::default());
                let pos2 = self.get_coords_for(Vec2f::new(x, yr.end));
                let v2 =
                    render::new_vertex(pos2, colors::rgba(180, 180, 180, 200), Vec2f::default());

                let mut text = render::create_text(&format!("{:.1}", x), font, font_size);

                render::render_line(window, &v1, &v2);
                render::render_text(window, &mut text, pos2 + Vec2f::new(2., 0.));

                x += xstep;
                iters += 1;
            }
        }
        if let Some(ystep) = self.config.grid_ystep {
            let mut y = yr.start;
            let mut iters = 0;
            while y <= yr.end && iters < 100 {
                let pos1 = self.get_coords_for(Vec2f::new(xr.start, y));
                let v1 =
                    render::new_vertex(pos1, colors::rgba(180, 180, 180, 200), Vec2f::default());
                let pos2 = self.get_coords_for(Vec2f::new(xr.end, y));
                let v2 =
                    render::new_vertex(pos2, colors::rgba(180, 180, 180, 200), Vec2f::default());

                let mut text = render::create_text(&format!("{:.1}", y), font, font_size);

                render::render_text(window, &mut text, pos1 + Vec2f::new(0., -2.));
                render::render_line(window, &v1, &v2);

                y += ystep;
                iters += 1;
            }
        }

        // Draw title
        if let Some(title) = self.config.title.as_ref() {
            let mut text = render::create_text(title, font, self.config.title_font_size);
            let bounds = render::get_text_local_bounds(&text);
            let pos =
                Vec2f::from(self.pos) + Vec2f::new(self.size.x as f32 - bounds.width - 2., 0.0);
            render::render_text(window, &mut text, pos);
        }

        // Draw line
        let drawn_points = self
            .data
            .points
            .iter()
            .filter(|Vec2f { x, y }| xr.contains(x) && yr.contains(y))
            .collect::<Vec<_>>();
        let mut vbuf = render::start_draw_linestrip(drawn_points.len());
        for &point in drawn_points {
            let vpos = self.get_coords_for(point);
            let col = self.get_color_for(point);
            let vertex = render::new_vertex(vpos, col, Vec2f::default());
            render::add_vertex(&mut vbuf, &vertex);
        }

        render::render_vbuf(window, &vbuf, &Transform2D::from_pos(self.pos.into()));
    }
}

impl Debug_Graph_View {
    pub fn new(config: Debug_Graph_View_Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    fn get_coords_for(&self, point: Vec2f) -> Vec2f {
        use crate::common::math::lerp;
        let w = self.data.x_range.end - self.data.x_range.start;
        let h = self.data.y_range.end - self.data.y_range.start;
        Vec2f::from(self.pos)
            + Vec2f::new(
                lerp(
                    0.0,
                    self.size.x as f32,
                    (point.x - self.data.x_range.start) / w,
                ),
                lerp(
                    0.0,
                    self.size.y as f32,
                    1.0 - (point.y - self.data.y_range.start) / h,
                ),
            )
    }

    fn get_color_for(&self, point: Vec2f) -> colors::Color {
        if let Some((lt, lc)) = self.config.low_threshold {
            if point.y < lt {
                return lc;
            }
        }
        if let Some((ht, hc)) = self.config.high_threshold {
            if point.y > ht {
                return hc;
            }
        }
        self.config.color
    }
}

impl Default for Debug_Graph {
    fn default() -> Self {
        Self {
            points: VecDeque::new(),
            x_range: 0.0..0.0,
            y_range: 0.0..0.0,
        }
    }
}

impl Debug_Graph {
    pub fn with_xy_range(x_range: Range<f32>, y_range: Range<f32>) -> Self {
        Debug_Graph {
            points: VecDeque::new(),
            x_range,
            y_range,
        }
    }

    pub fn add_point(&mut self, x: f32, y: f32) {
        self.points.push_back(Vec2f::new(x, y));
    }

    pub fn remove_points_before_x_range(&mut self) {
        while let Some(point) = self.points.front() {
            if point.x >= self.x_range.start {
                break;
            }
            self.points.pop_front();
        }
    }
}