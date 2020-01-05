use crate::core::common::rect::Rectf;
use crate::core::common::transform::Transform2D;
use crate::core::common::vector::Vec2f;
use crate::core::env::Env_Info;
use crate::gfx::render::Text;
use crate::gfx::render::{self, Paint_Properties};
use crate::gfx::window::Window_Handle;
use crate::resources::gfx;

pub struct Debug_Painter {
    rects: Vec<(Rectf, Transform2D, Paint_Properties)>,
    texts: Vec<(String, Vec2f, u16, Paint_Properties)>,
    font: gfx::Font_Handle,
}

const FONT_NAME: &str = "Hack-Regular.ttf";

impl Debug_Painter {
    pub fn new() -> Self {
        Debug_Painter {
            rects: vec![],
            texts: vec![],
            font: None,
        }
    }

    pub fn init(&mut self, gres: &mut gfx::Gfx_Resources, env: &Env_Info) {
        self.font = gres.load_font(&gfx::font_path(env, FONT_NAME));
    }

    pub fn add_rect(&mut self, rect: Rectf, transform: &Transform2D, props: &Paint_Properties) {
        self.rects.push((rect, *transform, *props));
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
    }

    pub fn draw(
        &self,
        window: &mut Window_Handle,
        gres: &mut gfx::Gfx_Resources,
        camera: &Transform2D,
    ) {
        assert!(self.font.is_some(), "Debug_Painter was not initialized!");

        for (rect, transform, props) in &self.rects {
            render::fill_color_rect_ws(window, props, *rect, transform, camera);
        }

        let font = self.font;
        for (text, world_pos, font_size, props) in &self.texts {
            let mut txt = Text::new(text, gres.get_font(font), (*font_size).into());
            txt.set_fill_color(props.color);
            txt.set_outline_thickness(props.border_thick);
            txt.set_outline_color(props.border_color);
            render::render_text(window, &mut txt, *world_pos);
        }
    }
}
