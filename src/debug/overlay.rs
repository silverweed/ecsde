use crate::core::common::colors::{self, Color};
use crate::core::common::vector::{to_framework_vec, Vec2f};
use crate::core::common::Maybe_Error;
use crate::core::env::Env_Info;
use crate::gfx;
use crate::gfx::window::Window_Handle;
use crate::resources;
use crate::resources::gfx::{Font_Handle, Gfx_Resources};

#[cfg(feature = "use-sfml")]
use sfml::graphics::Text;
#[cfg(feature = "use-sfml")]
use sfml::graphics::Transformable;

struct Debug_Line {
    pub text: String,
    pub color: Color,
}

#[derive(Copy, Clone)]
pub struct Debug_Overlay_Config {
    pub row_spacing: f32,
    pub font_size: u16,
    pub pad_x: f32,
    pub pad_y: f32,
}

pub enum Align {
    Begin,
    Middle,
    End,
}

pub struct Debug_Overlay {
    lines: Vec<Debug_Line>,
    font: Font_Handle,
    config: Debug_Overlay_Config,

    pub position: Vec2f,
    pub horiz_align: Align,
    pub vert_align: Align,
}

impl Debug_Overlay {
    pub fn new(config: Debug_Overlay_Config, font: Font_Handle) -> Debug_Overlay {
        Debug_Overlay {
            lines: vec![],
            font,
            config,
            position: Vec2f::new(0.0, 0.0),
            horiz_align: Align::Begin,
            vert_align: Align::Begin,
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn add_line(&mut self, line: &str) {
        self.lines.push(Debug_Line {
            text: String::from(line),
            color: colors::rgb(255, 255, 255),
        });
    }

    pub fn add_line_color(&mut self, line: &str, color: Color) {
        self.lines.push(Debug_Line {
            text: String::from(line),
            color,
        });
    }

    pub fn draw(&self, window: &mut Window_Handle, gres: &mut Gfx_Resources) {
        let font = self.font;
        let Debug_Overlay_Config {
            font_size,
            pad_x,
            pad_y,
            row_spacing,
            ..
        } = self.config;

        let mut texts = vec![];
        let mut max_row_height = 0f32;

        for line in self.lines.iter() {
            let Debug_Line { text, color } = line;
            let mut text = Text::new(text, gres.get_font(font), font_size.into());
            text.set_fill_color(&color);

            let txt_bounds = text.local_bounds();
            max_row_height = max_row_height.max(txt_bounds.height);

            texts.push((text, txt_bounds));
        }

        let position = self.position;
        let tot_height = max_row_height * texts.len() as f32;
        for (i, (text, bounds)) in texts.iter_mut().enumerate() {
            let pos = Vec2f::new(
                match self.horiz_align {
                    Align::Begin => pad_x,
                    Align::Middle => 0.5 * (pad_x - bounds.width),
                    Align::End => -(bounds.width + pad_x),
                },
                match self.vert_align {
                    Align::Begin => pad_y,
                    Align::Middle => 0.5 * (pad_y - tot_height),
                    Align::End => -(tot_height + pad_y),
                } + (i as f32) * (max_row_height + row_spacing),
            );
            text.set_position(to_framework_vec(position + pos));

            gfx::render::render_text(window, &text);
        }
    }
}
