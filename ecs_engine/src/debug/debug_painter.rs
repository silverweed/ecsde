use crate::core::common::colors::{self, Color};
use crate::core::common::rect::{Rect, Rectf};
use crate::core::common::shapes::Circle;
use crate::core::common::transform::Transform2D;
use crate::core::common::vector::Vec2f;
use crate::core::env::Env_Info;
use crate::gfx::render::Text;
use crate::gfx::render::{self, Paint_Properties};
use crate::gfx::window::Window_Handle;
use crate::prelude::*;
use crate::resources::gfx;

pub struct Debug_Painter {
    rects: Vec<(Vec2f, Transform2D, Paint_Properties)>,
    circles: Vec<(Circle, Paint_Properties)>,
    texts: Vec<(String, Vec2f, u16, Paint_Properties)>,
    font: gfx::Font_Handle,
}

const FONT_NAME: &str = "Hack-Regular.ttf";

impl Debug_Painter {
    pub fn new() -> Self {
        Debug_Painter {
            rects: vec![],
            circles: vec![],
            texts: vec![],
            font: None,
        }
    }

    pub fn init(&mut self, gres: &mut gfx::Gfx_Resources, env: &Env_Info) {
        self.font = gres.load_font(&gfx::font_path(env, FONT_NAME));
    }

    pub fn add_rect(&mut self, size: Vec2f, transform: &Transform2D, props: &Paint_Properties) {
        self.rects.push((size, *transform, *props));
    }

    pub fn add_circle(&mut self, circle: Circle, props: &Paint_Properties) {
        self.circles.push((circle, *props));
    }

    pub fn add_text(
        &mut self,
        text: &str,
        world_pos: Vec2f,
        font_size: u16,
        props: &Paint_Properties,
    ) {
        self.texts
            .push((String::from(text), world_pos, font_size, *props));
    }

    pub fn clear(&mut self) {
        self.rects.clear();
        self.circles.clear();
        self.texts.clear();
    }

    pub fn draw(
        &self,
        window: &mut Window_Handle,
        gres: &mut gfx::Gfx_Resources,
        camera: &Transform2D,
        tracer: Debug_Tracer,
    ) {
        trace!("debug_painter::draw", tracer);

        for (size, transform, props) in &self.rects {
            let rect = Rect::new(0., 0., size.x, size.y);
            trace!("debug_painter::fill_rect", tracer);
            render::fill_color_rect_ws(window, props, rect, transform, camera);
        }

        for (circle, props) in &self.circles {
            render::fill_color_circle_ws(window, props, *circle, camera);
        }

        let font = self.font;
        for (text, world_pos, font_size, props) in &self.texts {
            trace!("debug_painter::draw_text", tracer);
            let mut txt = Text::new(text, gres.get_font(font), (*font_size).into());
            txt.set_fill_color(props.color);
            txt.set_outline_thickness(props.border_thick);
            txt.set_outline_color(props.border_color);
            let transform = Transform2D::from_pos(*world_pos);
            render::render_text_ws(window, &txt, &transform, camera);
        }
    }
}
