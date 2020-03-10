use super::element::Debug_Element;
use crate::common::colors::{self, Color};
use crate::common::rect::Rect;
use crate::common::vector::Vec2f;
use crate::gfx;
use crate::gfx::align::Align;
use crate::gfx::window::Window_Handle;
use crate::resources::gfx::{Font_Handle, Gfx_Resources};

struct Debug_Line {
    pub text: String,
    pub color: Color,
    // (fill color, horizontal fill ratio)
    pub bg_rect_fill: Option<(Color, f32)>, // @Cleanup: this is not very pretty
}

#[derive(Copy, Clone, Default)]
pub struct Debug_Overlay_Config {
    pub row_spacing: f32,
    pub font_size: u16,
    pub pad_x: f32,
    pub pad_y: f32,
    pub background: Color,
    pub font: Font_Handle,
    pub horiz_align: Align,
    pub vert_align: Align,
}

#[derive(Default)]
pub struct Debug_Overlay {
    lines: Vec<Debug_Line>,

    pub config: Debug_Overlay_Config,
    pub position: Vec2f,
}

impl Debug_Element for Debug_Overlay {
    fn draw(&self, window: &mut Window_Handle, gres: &mut Gfx_Resources) {
        trace!("overlay::draw");

        if self.lines.is_empty() {
            return;
        }

        let Debug_Overlay_Config {
            font,
            font_size,
            pad_x,
            pad_y,
            row_spacing,
            horiz_align,
            vert_align,
            ..
        } = self.config;

        let mut texts = Vec::with_capacity(self.lines.len());
        let mut max_row_width = 0f32;
        let mut max_row_height = 0f32;

        for line in self.lines.iter() {
            let Debug_Line { text, color, .. } = line;
            let text = gfx::render::create_text(text, gres.get_font(font), font_size);

            let txt_bounds = gfx::render::get_text_local_bounds(&text);
            max_row_width = max_row_width.max(txt_bounds.width);
            max_row_height = max_row_height.max(txt_bounds.height);

            texts.push((text, *color, txt_bounds));
        }

        let position = self.position;
        let n_texts_f = texts.len() as f32;
        let tot_height = max_row_height * n_texts_f + row_spacing * (n_texts_f - 1.0);

        // Draw background

        gfx::render::fill_color_rect(
            window,
            self.config.background,
            Rect::new(
                position.x + horiz_align.aligned_pos(0.0, 2.0 * pad_x + max_row_width),
                position.y + vert_align.aligned_pos(0.0, 2.0 * pad_y + tot_height),
                2.0 * pad_x + max_row_width,
                2.0 * pad_y + tot_height,
            ),
        );

        // Draw bg rects
        for (i, (bg_col, bg_fill_ratio)) in self
            .lines
            .iter()
            .enumerate()
            .filter_map(|(i, line)| Some((i, line.bg_rect_fill?)))
        {
            let pos = position
                + Vec2f::new(
                    horiz_align.aligned_pos(pad_x, max_row_width),
                    vert_align.aligned_pos(pad_y, tot_height)
                        + (i as f32) * (max_row_height + row_spacing),
                );
            let rect = Rect::new(pos.x, pos.y, bg_fill_ratio * max_row_width, max_row_height);
            gfx::render::fill_color_rect(window, bg_col, rect);
        }

        // Draw texts
        for (i, (text, color, bounds)) in texts.iter_mut().enumerate() {
            let pos = Vec2f::new(
                horiz_align.aligned_pos(pad_x, bounds.width),
                vert_align.aligned_pos(pad_y, tot_height)
                    + (i as f32) * (max_row_height + row_spacing),
            );
            gfx::render::render_text(window, text, *color, position + pos);
        }
    }
}

impl Debug_Overlay {
    pub fn new(config: Debug_Overlay_Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn add_line(&mut self, line: &str) {
        self.lines.push(Debug_Line {
            text: String::from(line),
            color: colors::WHITE,
            bg_rect_fill: None,
        });
    }

    pub fn add_line_color(&mut self, line: &str, color: Color) {
        self.lines.push(Debug_Line {
            text: String::from(line),
            color,
            bg_rect_fill: None,
        });
    }

    pub fn add_line_color_with_bg_fill(
        &mut self,
        line: &str,
        color: Color,
        bg_rect_fill: (Color, f32),
    ) {
        self.lines.push(Debug_Line {
            text: String::from(line),
            color,
            bg_rect_fill: Some(bg_rect_fill),
        });
    }
}
