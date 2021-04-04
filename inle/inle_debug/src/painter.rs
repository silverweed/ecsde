use inle_common::colors::Color;
use inle_common::paint_props::Paint_Properties;
use inle_core::env::Env_Info;
use inle_gfx::render;
use inle_gfx::render::Vertex_Buffer_Quads;
use inle_gfx::render_window::Render_Window_Handle;
use inle_math::angle::rad;
use inle_math::rect::Rect;
use inle_math::shapes::{Arrow, Circle, Line};
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use inle_resources::gfx;

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
    pub fn new() -> Self {
        Debug_Painter {
            rects: vec![],
            circles: vec![],
            texts: vec![],
            arrows: vec![],
            lines: vec![],
            font: None,
        }
    }

    pub fn init(&mut self, gres: &mut gfx::Gfx_Resources, env: &Env_Info) {
        self.font = gres.load_font(&gfx::font_path(env, FONT_NAME));
    }

    pub fn add_rect<T>(&mut self, size: Vec2f, transform: &Transform2D, props: T)
    where
        T: Into<Paint_Properties>,
    {
        self.rects.push((size, *transform, props.into()));
    }

    pub fn add_circle<T>(&mut self, circle: Circle, props: T)
    where
        T: Into<Paint_Properties>,
    {
        self.circles.push((circle, props.into()));
    }

    pub fn add_arrow<T>(&mut self, arrow: Arrow, props: T)
    where
        T: Into<Paint_Properties>,
    {
        self.arrows.push((arrow, props.into()));
    }

    pub fn add_text<T>(&mut self, text: &str, world_pos: Vec2f, font_size: u16, props: T)
    where
        T: Into<Paint_Properties>,
    {
        self.texts
            .push((String::from(text), world_pos, font_size, props.into()));
    }

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

        for (size, transform, props) in &self.rects {
            let rect = Rect::new(0., 0., size.x, size.y);
            trace!("painter::fill_rect");
            render::render_rect_ws(window, rect, *props, transform, camera);
        }

        for (circle, props) in &self.circles {
            render::render_circle_ws(window, *circle, *props, camera);
        }

        for (arrow, props) in &self.arrows {
            trace!("painter::draw_arrow");
            draw_arrow(window, arrow, props, camera);
        }

        for (line, props) in &self.lines {
            trace!("painter::draw_lines");
            draw_line(window, line, props, camera);
        }

        let font = gres.get_font(self.font);
        for (text, world_pos, font_size, props) in &self.texts {
            trace!("painter::draw_text");
            let aa_adj_scale = if *font_size < 10 {
                4
            } else if *font_size < 20 {
                2
            } else {
                1
            };
            let aa_adj_inv_scale = 1. / (aa_adj_scale as f32);
            let mut txt = render::create_text(text, font, aa_adj_scale * *font_size);
            let transform = Transform2D::from_pos_rot_scale(
                *world_pos,
                rad(0.),
                v2!(aa_adj_inv_scale, aa_adj_inv_scale),
            );
            render::render_text_ws(window, &mut txt, *props, &transform, camera);
        }
    }
}

fn draw_line(
    window: &mut Render_Window_Handle,
    line: &Line,
    props: &Paint_Properties,
    camera: &Transform2D,
) {
    let mut vbuf = render::start_draw_quads_temp(window, 1);
    let direction = line.to - line.from;
    draw_line_internal(
        window,
        &mut vbuf,
        direction.magnitude(),
        line.thickness,
        props,
    );

    let rot = rad(direction.y.atan2(direction.x));
    let transform = Transform2D::from_pos_rot_scale(line.from, rot, Vec2f::new(1., 1.));
    render::render_vbuf_ws(window, &vbuf, &transform, camera);
}

fn draw_arrow(
    window: &mut Render_Window_Handle,
    arrow: &Arrow,
    props: &Paint_Properties,
    camera: &Transform2D,
) {
    let mut vbuf = render::start_draw_quads_temp(window, 2);
    let magnitude = arrow.direction.magnitude();
    draw_line_internal(window, &mut vbuf, magnitude, arrow.thickness, props);

    // Draw arrow tip
    {
        let v5 = render::new_vertex(
            Vec2f::new(magnitude - arrow.arrow_size * 0.5, -arrow.arrow_size * 0.5),
            props.color,
            Vec2f::default(),
        );
        let v6 = render::new_vertex(
            Vec2f::new(magnitude - arrow.arrow_size * 0.5, arrow.arrow_size * 0.5),
            props.color,
            Vec2f::default(),
        );
        let v7 = render::new_vertex(Vec2f::new(magnitude, 0.), props.color, Vec2f::default());

        render::add_quad(window, &mut vbuf, &v5, &v7, &v7, &v6);
    }

    let rot = rad(arrow.direction.y.atan2(arrow.direction.x));
    let transform = Transform2D::from_pos_rot_scale(arrow.center, rot, Vec2f::new(1., 1.));

    render::render_vbuf_ws(window, &vbuf, &transform, camera);
}

fn draw_line_internal(
    window: &mut Render_Window_Handle,
    vbuf: &mut Vertex_Buffer_Quads,
    length: f32,
    thickness: f32,
    props: &Paint_Properties,
) {
    let v1 = render::new_vertex(
        Vec2f::new(0., -thickness * 0.5),
        props.color,
        Vec2f::default(),
    );
    let v2 = render::new_vertex(
        Vec2f::new(length, -thickness * 0.5),
        props.color,
        Vec2f::default(),
    );
    let v3 = render::new_vertex(
        Vec2f::new(length, thickness * 0.5),
        props.color,
        Vec2f::default(),
    );
    let v4 = render::new_vertex(
        Vec2f::new(0., thickness * 0.5),
        props.color,
        Vec2f::default(),
    );

    render::add_quad(window, vbuf, &v1, &v2, &v3, &v4);
}
