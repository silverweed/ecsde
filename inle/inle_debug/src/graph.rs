use super::element::{Debug_Element, Draw_Args, Update_Args, Update_Res};
use inle_common::colors;
use inle_common::paint_props::Paint_Properties;
use inle_common::stringid::String_Id;
use inle_common::variant::Variant;
use inle_gfx::render;
use inle_input::mouse;
use inle_math::rect::Rect;
use inle_math::shapes::Circle;
use inle_math::transform::Transform2D;
use inle_math::vector::{Vec2f, Vec2u};
use inle_resources::gfx::Font_Handle;
use std::collections::{HashMap, VecDeque};
use std::convert::TryFrom;
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
    pub hoverable: bool,
    pub show_average: bool,
}

#[derive(Default)]
pub struct Debug_Graph_View {
    pub data: Debug_Graph,
    pub pos: Vec2u,
    pub size: Vec2u,
    pub config: Debug_Graph_View_Config,

    // goes from 0 to data.points.len() - 1
    pub hovered_point: Option<usize>,
    pub selected_point: Option<usize>,
}

/// Note: for simplicify, the graph assumes points are added in x-sorted order.
pub struct Debug_Graph {
    pub points: VecDeque<Vec2f>,
    pub x_range: Range<f32>,
    pub max_y_value: Option<f32>,
    pub min_y_value: Option<f32>,

    points_metadata: VecDeque<HashMap<String_Id, Variant>>,
}

impl Debug_Element for Debug_Graph_View {
    fn update(
        &mut self,
        Update_Args {
            window,
            input_state,
            ..
        }: Update_Args,
    ) -> Update_Res {
        trace!("debug::graph::update");

        if !self.config.hoverable {
            return Update_Res::Stay_Enabled;
        }

        let mpos = Vec2f::from(mouse::mouse_pos_in_window(
            window,
            &input_state.raw.mouse_state,
        ));
        let rect = Rect::new(self.pos.x, self.pos.y, self.size.x, self.size.y);
        self.hovered_point = None;
        if rect.contains(mpos) {
            // Find out which point is being hovered
            // @Speed!
            let xr = &self.data.x_range;
            let yr = self.y_range();
            let drawn_points = self
                .data
                .points
                .iter()
                .filter(|Vec2f { x, y }| xr.contains(x) && yr.contains(y));
            for (i, point) in drawn_points.enumerate() {
                let point_pos = self.get_coords_for(*point);
                if point_pos.x >= mpos.x as f32 {
                    self.hovered_point = Some(i);
                    break;
                }
            }
        }

        if mouse::mouse_went_down(&input_state.raw.mouse_state, mouse::Mouse_Button::Left) {
            self.selected_point = self.hovered_point;
        }

        Update_Res::Stay_Enabled
    }

    fn draw(
        &self,
        Draw_Args {
            window,
            gres,
            input_state,
            ..
        }: Draw_Args,
    ) {
        trace!("debug::graph::draw");

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
        let mut vbuf = render::start_draw_linestrip_temp(window, drawn_points.len() as _);
        let mut avg = 0.0;
        for (i, &&point) in drawn_points.iter().enumerate() {
            let vpos = self.get_coords_for(point);
            let col = self.get_color_for(point);
            let vertex = render::new_vertex(vpos, col, Vec2f::default());
            avg += point.y;
            render::add_vertex(&mut vbuf, &vertex);

            // Draw selection line
            if let Some(x) = self.selected_point {
                if i == x {
                    let rpos = pos + v2!(vpos.x, 0.) + v2!(-1., -1.);
                    let rect = Rect::new(rpos.x, rpos.y, 2., self.size.y as f32 + 2.);
                    render::render_rect(
                        window,
                        rect,
                        Paint_Properties {
                            color: colors::WHITE,
                            border_color: colors::WHITE,
                            border_thick: 1.,
                            ..Default::default()
                        },
                    );
                    render::render_circle(
                        window,
                        Circle {
                            center: pos + vpos,
                            radius: 4.0,
                        },
                        colors::rgb(255, 50, 10),
                    );
                }
            }

            // Draw hover line
            if let Some(x) = self.hovered_point {
                if i == x {
                    let color = colors::WHITE;
                    let mpos = Vec2f::from(mouse::mouse_pos_in_window(
                        window,
                        &input_state.raw.mouse_state,
                    ));
                    let v1 = render::new_vertex(pos + v2!(mpos.x, 0.0), color, Vec2f::default());
                    let v2 = render::new_vertex(
                        pos + v2!(mpos.x, self.size.y as f32),
                        color,
                        Vec2f::default(),
                    );
                    let circle_col = colors::rgb(10, 255, 200);
                    render::render_line(window, &v1, &v2);
                    render::render_circle(
                        window,
                        Circle {
                            center: pos + vpos,
                            radius: 4.0,
                        },
                        circle_col,
                    );
                    let mut text = render::create_text(
                        &format!("{:.2}", self.data.points[x].y),
                        font,
                        (1.5 * self.config.label_font_size as f32) as _,
                    );
                    render::render_text(
                        window,
                        &mut text,
                        circle_col,
                        pos + vpos + v2!(30.0, -30.0),
                    );
                }
            }
        }

        // Draw average
        if self.config.show_average {
            avg /= drawn_points.len() as f32;
            let avg_line_col = colors::rgb(149, 206, 255);
            let start = render::new_vertex(
                pos + self.get_coords_for(v2!(self.data.x_range.start, avg)),
                avg_line_col,
                v2!(0.0, 0.0),
            );
            let end = render::new_vertex(
                pos + self.get_coords_for(v2!(self.data.x_range.end, avg)),
                avg_line_col,
                v2!(0.0, 0.0),
            );
            render::render_line(window, &start, &end);
            let mut text = render::create_text(
                &format!("{:.2}", avg),
                font,
                (1.5 * self.config.label_font_size as f32) as _,
            );
            render::render_text(
                window,
                &mut text,
                avg_line_col,
                pos + self.get_coords_for(v2!(self.data.x_range.start, avg)) + v2!(40.0, -25.0),
            );
        }

        render::render_vbuf(window, &vbuf, &Transform2D::from_pos(self.pos.into()));
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Debug_Graph_Point {
    pub value: Vec2f,
    pub index: usize, // useful for retrieving metadata
}

impl Debug_Graph_View {
    pub fn new(config: Debug_Graph_View_Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    pub fn get_selected_point(&self) -> Option<Debug_Graph_Point> {
        self.selected_point.map(|i| Debug_Graph_Point {
            value: self.data.points[i],
            index: i,
        })
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
        use inle_math::math::lerp;
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
            points_metadata: VecDeque::new(),
            x_range: 0.0..0.0,
            max_y_value: None,
            min_y_value: None,
        }
    }
}

impl Debug_Graph {
    pub fn add_point(&mut self, x: f32, y: f32, metadata: &[(String_Id, Variant)]) {
        self.min_y_value = Some(self.min_y_value.unwrap_or(y).min(y));
        self.max_y_value = Some(self.max_y_value.unwrap_or(y).max(y));
        self.points.push_back(Vec2f::new(x, y));
        self.points_metadata
            .push_back(metadata.iter().cloned().collect());
    }

    pub fn remove_points_before_x_range(&mut self) {
        while let Some(point) = self.points.front() {
            if point.x >= self.x_range.start {
                break;
            }
            self.points.pop_front();
        }
    }

    pub fn get_point_metadata<T>(&self, point_index: usize, key: String_Id) -> Option<T>
    where
        T: TryFrom<Variant>,
    {
        self.points_metadata[point_index].get(&key).map(|val| {
            T::try_from(val.clone()).unwrap_or_else(|_| {
                fatal!(
                    "Failed to convert metadata {} for point {} into {:?} (was {:?})",
                    key,
                    point_index,
                    std::any::type_name::<T>(),
                    val
                )
            })
        })
    }
}

pub fn add_point_and_scroll(
    graph: &mut Debug_Graph_View,
    now: std::time::Duration,
    time_limit: f32,
    point: f32,
) {
    add_point_and_scroll_with_metadata(graph, now, time_limit, point, &[]);
}

pub fn add_point_and_scroll_with_metadata(
    graph: &mut Debug_Graph_View,
    now: std::time::Duration,
    time_limit: f32,
    point: f32,
    metadata: &[(String_Id, Variant)],
) {
    let now = now.as_secs_f32();
    graph.data.x_range.end = now;
    if graph.data.x_range.end - graph.data.x_range.start > time_limit {
        graph.data.x_range.start = graph.data.x_range.end - time_limit;
    }
    graph.data.add_point(now, point, metadata);
    graph.data.remove_points_before_x_range();
}
