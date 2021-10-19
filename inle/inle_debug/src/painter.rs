use inle_common::colors::Color;
use inle_common::paint_props::Paint_Properties;
use inle_core::env::Env_Info;
use inle_gfx::render::{self, Font, Vertex_Buffer_Triangles};
use inle_gfx::render_window::Render_Window_Handle;
use inle_math::angle::rad;
use inle_math::rect::{Rect, Rectf};
use inle_math::shapes::{Arrow, Circle, Line};
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use inle_resources::gfx;
use std::convert::TryFrom;

#[derive(Default)]
pub struct Debug_Painter {
    rects: Vec<(Vec2f, Transform2D, Paint_Properties)>,
    circles: Vec<(Circle, Paint_Properties)>,
    texts: Vec<(String, Vec2f, u16, Paint_Properties)>,
    arrows: Vec<(Arrow, Paint_Properties)>,
    lines: Vec<(Line, Paint_Properties)>,
    font: gfx::Font_Handle,
}

const FONT_NAME: &str = "Hack-Regular.ttf";

impl Debug_Painter {
    pub fn init(&mut self, gres: &mut gfx::Gfx_Resources, env: &Env_Info) {
        self.font = gres.load_font(&gfx::font_path(env, FONT_NAME));
    }

    pub fn add_rect<T>(&mut self, size: Vec2f, transform: &Transform2D, props: T)
    where
        T: Into<Paint_Properties>,
    {
        self.rects.push((size, *transform, props.into()));
    }

    // @Consistency: we should allow to pass the transform here like in add_rect
    pub fn add_circle<T>(&mut self, circle: Circle, props: T)
    where
        T: Into<Paint_Properties>,
    {
        self.circles.push((circle, props.into()));
    }

    // @Consistency: we should allow to pass the transform here like in add_rect
    pub fn add_arrow<T>(&mut self, arrow: Arrow, props: T)
    where
        T: Into<Paint_Properties>,
    {
        self.arrows.push((arrow, props.into()));
    }

    // @Consistency: we should allow to pass the transform here like in add_rect
    pub fn add_text<T>(&mut self, text: &str, world_pos: Vec2f, font_size: u16, props: T)
    where
        T: Into<Paint_Properties>,
    {
        self.texts
            .push((String::from(text), world_pos, font_size, props.into()));
    }

    // @Consistency: we should allow to pass the transform here like in add_rect
    pub fn add_shaded_text_with_shade_distance<T>(
        &mut self,
        text: &str,
        world_pos: Vec2f,
        font_size: u16,
        props: T,
        shade_color: Color,
        shade_distance: Vec2f,
    ) where
        T: Into<Paint_Properties>,
    {
        let props = props.into();
        let shade_props = Paint_Properties {
            color: shade_color,
            ..props
        };
        self.texts.push((
            String::from(text),
            world_pos + shade_distance,
            font_size,
            shade_props,
        ));
        self.texts
            .push((String::from(text), world_pos, font_size, props));
    }

    // @Consistency: we should allow to pass the transform here like in add_rect
    pub fn add_shaded_text<T>(
        &mut self,
        text: &str,
        world_pos: Vec2f,
        font_size: u16,
        props: T,
        shade_color: Color,
    ) where
        T: Into<Paint_Properties>,
    {
        self.add_shaded_text_with_shade_distance(
            text,
            world_pos,
            font_size,
            props,
            shade_color,
            v2!(1., 1.),
        );
    }

    // @Consistency: maybe we should allow to pass the transform here like in add_rect
    pub fn add_line<T>(&mut self, line: Line, props: T)
    where
        T: Into<Paint_Properties>,
    {
        self.lines.push((line, props.into()));
    }

    pub fn clear(&mut self) {
        self.rects.clear();
        self.circles.clear();
        self.texts.clear();
        self.arrows.clear();
        self.lines.clear();
    }

    pub fn draw(
        &self,
        window: &mut Render_Window_Handle,
        gres: &mut gfx::Gfx_Resources,
        camera: &Transform2D,
    ) {
        trace!("painter::draw");

        let visible_viewport = inle_win::window::get_camera_viewport(window, camera);

        let tot_circle_points_needed = 3 * self
            .circles
            .iter()
            .map(|(_, props)| props.point_count as usize)
            .sum::<usize>();
        let tot_triangles = tot_circle_points_needed
            + 2 * self.rects.len()
            + 3 * self.arrows.len()
            + 2 * self.lines.len();

        if tot_triangles > 0 {
            let mut vbuf =
                render::start_draw_triangles_temp(window, u32::try_from(tot_triangles).unwrap());

            for (size, transform, props) in &self.rects {
                draw_rect_internal(&mut vbuf, *size, transform, props, &visible_viewport);
            }

            for (circle, props) in &self.circles {
                draw_circle_internal(&mut vbuf, circle, props, &visible_viewport);
            }

            for (arrow, props) in &self.arrows {
                draw_arrow(&mut vbuf, arrow, props);
            }

            for (line, props) in &self.lines {
                let direction = line.to - line.from;
                draw_line_internal(&mut vbuf, line.from, direction, line.thickness, props);
            }

            render::render_vbuf_ws(window, &vbuf, &Transform2D::default(), camera);
        }

        let font = gres.get_font(self.font);
        for (text, world_pos, font_size, props) in &self.texts {
            draw_text(
                window,
                text,
                *world_pos,
                font,
                *font_size,
                props,
                camera,
                &visible_viewport,
            );
        }
    }
}

fn draw_arrow(vbuf: &mut Vertex_Buffer_Triangles, arrow: &Arrow, props: &Paint_Properties) {
    trace!("painter::draw_arrow");

    let (magnitude, m) =
        draw_line_internal(vbuf, arrow.center, arrow.direction, arrow.thickness, props);

    // Draw arrow tip
    {
        let v5 = render::new_vertex(
            m * Vec2f::new(magnitude - arrow.arrow_size * 0.5, -arrow.arrow_size * 0.5),
            props.color,
            Vec2f::default(),
        );
        let v6 = render::new_vertex(
            m * Vec2f::new(magnitude - arrow.arrow_size * 0.5, arrow.arrow_size * 0.5),
            props.color,
            Vec2f::default(),
        );
        let v7 = render::new_vertex(m * Vec2f::new(magnitude, 0.), props.color, Vec2f::default());

        render::add_triangle(vbuf, &v5, &v7, &v6);
    }
}

fn draw_line_internal(
    vbuf: &mut Vertex_Buffer_Triangles,
    start: Vec2f,
    direction: Vec2f,
    thickness: f32,
    props: &Paint_Properties,
) -> (f32, Transform2D) {
    trace!("painter::draw_line_internal");

    let length = direction.magnitude();
    let rot = rad(direction.y.atan2(direction.x));
    let m = Transform2D::from_pos_rot_scale(start, rot, v2!(1., 1.));

    let v1 = render::new_vertex(
        m * Vec2f::new(0., -thickness * 0.5),
        props.color,
        Vec2f::default(),
    );
    let v2 = render::new_vertex(
        m * Vec2f::new(length, -thickness * 0.5),
        props.color,
        Vec2f::default(),
    );
    let v3 = render::new_vertex(
        m * Vec2f::new(length, thickness * 0.5),
        props.color,
        Vec2f::default(),
    );
    let v4 = render::new_vertex(
        m * Vec2f::new(0., thickness * 0.5),
        props.color,
        Vec2f::default(),
    );

    render::add_triangle(vbuf, &v1, &v2, &v3);
    render::add_triangle(vbuf, &v3, &v4, &v1);

    (length, m)
}

fn draw_rect_internal(
    vbuf: &mut Vertex_Buffer_Triangles,
    size: Vec2f,
    transform: &Transform2D,
    props: &Paint_Properties,
    visible_viewport: &Rectf,
) {
    trace!("painter::draw_rect_internal");

    // @Incomplete: the rect aabb is not considering rotation!
    if inle_math::rect::rects_intersection(
        visible_viewport,
        &Rect::from_topleft_size(transform.position(), size * transform.scale()),
    )
    .is_none()
    {
        return;
    }

    let m = *transform;
    let v1 = render::new_vertex(m * v2!(0., 0.), props.color, v2!(0., 0.));
    let v2 = render::new_vertex(m * v2!(size.x, 0.), props.color, v2!(0., 0.));
    let v3 = render::new_vertex(m * size, props.color, v2!(0., 0.));
    let v4 = render::new_vertex(m * v2!(0., size.y), props.color, v2!(0., 0.));

    render::add_triangle(vbuf, &v1, &v2, &v3);
    render::add_triangle(vbuf, &v3, &v4, &v1);
}

fn draw_circle_internal(
    vbuf: &mut Vertex_Buffer_Triangles,
    circle: &Circle,
    props: &Paint_Properties,
    visible_viewport: &Rectf,
) {
    trace!("painter::draw_circle_internal");

    let aabb = Rect::new(
        circle.center.x - circle.radius,
        circle.center.y - circle.radius,
        2. * circle.radius,
        2. * circle.radius,
    );
    if inle_math::rect::rects_intersection(visible_viewport, &aabb).is_none() {
        return;
    }

    let angle_step = std::f32::consts::TAU / props.point_count as f32;
    let cos_and_sin = (0..props.point_count)
        .map(|i| {
            let (s, c) = (i as f32 * angle_step).sin_cos();
            v2!(c, s)
        })
        .collect::<Vec<_>>();

    for i in 0..props.point_count {
        let pt = cos_and_sin[i as usize];
        let pt_next = cos_and_sin[((i + 1) % props.point_count) as usize];

        let v1 = render::new_vertex(circle.center, props.color, v2!(0., 0.));
        let v2 = render::new_vertex(circle.center + pt * circle.radius, props.color, v2!(0., 0.));
        let v3 = render::new_vertex(
            circle.center + pt_next * circle.radius,
            props.color,
            v2!(0., 0.),
        );
        render::add_triangle(vbuf, &v1, &v2, &v3);
    }
}

fn draw_text(
    window: &mut Render_Window_Handle,
    text: &str,
    world_pos: Vec2f,
    font: &Font,
    font_size: u16,
    props: &Paint_Properties,
    camera: &Transform2D,
    visible_viewport: &Rectf,
) {
    // @Speed: batch the texts!
    trace!("painter::draw_text");

    let aa_adj_scale = if font_size < 10 {
        4
    } else if font_size < 20 {
        2
    } else {
        1
    };
    let aa_adj_inv_scale = 1. / (aa_adj_scale as f32);

    let mut txt = render::create_text(window, text, font, aa_adj_scale * font_size);
    let transform = Transform2D::from_pos_rot_scale(
        world_pos,
        rad(0.),
        v2!(aa_adj_inv_scale, aa_adj_inv_scale),
    );

    let text_size = render::get_text_size(&txt);
    let text_aabb = Rect::from_topleft_size(transform.position(), text_size);
    if inle_math::rect::rects_intersection(visible_viewport, &text_aabb).is_none() {
        return;
    }

    render::render_text_ws(window, &mut txt, *props, &transform, camera);
}
