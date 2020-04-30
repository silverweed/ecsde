use super::element::Debug_Element;
use crate::alloc::temp;
use crate::common::colors;
use crate::common::rect::Rect;
use crate::common::transform::Transform2D;
use crate::common::vector::{Vec2f, Vec2u};
use crate::core::time;
use crate::gfx::render;
use crate::gfx::window::Window_Handle;
use crate::resources::gfx::{Font_Handle, Gfx_Resources};
use std::collections::VecDeque;
use std::ops::Range;

#[derive(Copy, Clone, Debug)]
pub enum Grid_Step {
    Fixed_Step(f32),
    Fixed_Subdivs(usize),
}

#[derive(Default, Clone, Debug)]
pub struct Debug_Graph_View_Config {
    pub grid_xstep: Option<Grid_Step>,
    pub grid_ystep: Option<Grid_Step>,
    pub font: Font_Handle,
    pub label_font_size: u16,
    pub title: Option<String>,
    pub title_font_size: u16,
    pub color: colors::Color,
    // If value is > than this, use this other color
    pub low_threshold: Option<(f32, colors::Color)>,
    // If value is < than this, use this other color
    pub high_threshold: Option<(f32, colors::Color)>,
    pub fixed_y_range: Option<Range<f32>>,
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
    max_y_value: Option<f32>,
    min_y_value: Option<f32>,
}

impl Debug_Element for Debug_Graph_View {
    fn draw(
        &self,
        window: &mut Window_Handle,
        gres: &mut Gfx_Resources,
        _frame_alloc: &mut temp::Temp_Allocator,
    ) {
        trace!("graph::draw");

        // Draw background
        {
            let Vec2u { x, y } = self.pos;
            let Vec2u { x: w, y: h } = self.size;
            render::render_rect(window, Rect::new(x, y, w, h), colors::rgba(0, 0, 0, 200));
        }

        let xr = &self.data.x_range;
        let yr = self.y_range();

        // Draw grid
        let font = gres.get_font(self.config.font);
        let font_size = self.config.label_font_size;
        let pos = Vec2f::from(self.pos);
        if let Some(xstep) = self.config.grid_xstep {
            let xstep = match xstep {
                Grid_Step::Fixed_Step(step) => step,
                Grid_Step::Fixed_Subdivs(sub) => (xr.end - xr.start) / sub as f32,
            };
            let mut x = xr.start;
            let mut iters = 0;
            while x <= xr.end && iters < 100 {
                let pos1 = pos + self.get_coords_for(Vec2f::new(x, yr.start));
                let v1 =
                    render::new_vertex(pos1, colors::rgba(180, 180, 180, 200), Vec2f::default());
                let pos2 = pos + self.get_coords_for(Vec2f::new(x, yr.end));
                let v2 =
                    render::new_vertex(pos2, colors::rgba(180, 180, 180, 200), Vec2f::default());

                let mut text = render::create_text(&format!("{:.1}", x), font, font_size);

                render::render_line(window, &v1, &v2);
                // Skip first x label, or it overlaps with first y label
                if iters > 0 {
                    render::render_text(
                        window,
                        &mut text,
                        colors::WHITE,
                        pos2 + Vec2f::new(2., 0.),
                    );
                }

                x += xstep;
                iters += 1;
            }
        }
        if let Some(ystep) = self.config.grid_ystep {
            let ystep = match ystep {
                Grid_Step::Fixed_Step(step) => step,
                Grid_Step::Fixed_Subdivs(sub) => (yr.end - yr.start) / sub as f32,
            };
            let mut y = yr.start;
            let mut iters = 0;
            while y <= yr.end && iters < 100 {
                let pos1 = pos + self.get_coords_for(Vec2f::new(xr.start, y));
                let v1 =
                    render::new_vertex(pos1, colors::rgba(180, 180, 180, 200), Vec2f::default());
                let pos2 = pos + self.get_coords_for(Vec2f::new(xr.end, y));
                let v2 =
                    render::new_vertex(pos2, colors::rgba(180, 180, 180, 200), Vec2f::default());

                let mut text = render::create_text(&format!("{:.2}", y), font, font_size);

                render::render_line(window, &v1, &v2);
                render::render_text(window, &mut text, colors::WHITE, pos1 + Vec2f::new(0., -2.));

                y += ystep;
                iters += 1;
            }
        }

        // Draw title
        if let Some(title) = self.config.title.as_ref() {
            let mut text = render::create_text(title, font, self.config.title_font_size);
            let size = render::get_text_size(&text);
            let pos = Vec2f::from(self.pos) + Vec2f::new(self.size.x as f32 - size.x - 2., 0.0);
            render::render_text(window, &mut text, colors::WHITE, pos);
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

    fn y_range(&self) -> Range<f32> {
        if let Some(y_range) = &self.config.fixed_y_range {
            y_range.clone()
        } else {
            let min_y = self.data.min_y_value.unwrap_or(0.);
            let max_y = self.data.max_y_value.unwrap_or(0.);
            let diff = max_y - min_y;
            (min_y - diff * 0.1)..(max_y + diff * 0.1)
        }
    }

    fn get_coords_for(&self, point: Vec2f) -> Vec2f {
        use crate::common::math::lerp;
        let w = self.data.x_range.end - self.data.x_range.start;
        let yr = self.y_range();
        let h = yr.end - yr.start;
        Vec2f::new(
            lerp(
                0.0,
                self.size.x as f32,
                (point.x - self.data.x_range.start) / w,
            ),
            lerp(0.0, self.size.y as f32, 1.0 - (point.y - yr.start) / h),
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
            max_y_value: None,
            min_y_value: None,
        }
    }
}

impl Debug_Graph {
    pub fn add_point(&mut self, x: f32, y: f32) {
        self.min_y_value = Some(self.min_y_value.unwrap_or(y).min(y));
        self.max_y_value = Some(self.max_y_value.unwrap_or(y).max(y));
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

pub fn graph_add_point_and_scroll(
    graph: &mut Debug_Graph_View,
    time: &time::Time,
    time_limit: f32,
    point: f32,
) {
    let now = time.get_real_time().as_secs_f32();
    graph.data.x_range.end = now;
    if graph.data.x_range.end - graph.data.x_range.start > time_limit {
        graph.data.x_range.start = graph.data.x_range.end - time_limit;
    }
    graph.data.add_point(now, point);
}
