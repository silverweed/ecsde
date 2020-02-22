use crate::core::common::colors::{self, Color};
use crate::core::common::rect::Rect;
use crate::core::common::vector::Vec2f;
use crate::gfx;
use crate::gfx::align::Align;
use crate::gfx::window::Window_Handle;
use crate::resources::gfx::{Font_Handle, Gfx_Resources};

struct Debug_Line {
    pub text: String,
    pub color: Color,
    pub bg_rect_fill: Option<(Color, f32)>, // @Cleanup: this is not very pretty
}

#[derive(Copy, Clone)]
pub struct Debug_Overlay_Config {
    pub row_spacing: f32,
    pub font_size: u16,
    pub pad_x: f32,
    pub pad_y: f32,
    pub background: Color,
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

    pub fn draw(&self, window: &mut Window_Handle, gres: &mut Gfx_Resources) {
        if self.lines.is_empty() {
            return;
        }

        let font = self.font;
        let Debug_Overlay_Config {
            font_size,
            pad_x,
            pad_y,
            row_spacing,
            ..
        } = self.config;

        let mut texts = Vec::with_capacity(self.lines.len());
        let mut max_row_width = 0f32;
        let mut max_row_height = 0f32;

        for line in self.lines.iter() {
            let Debug_Line { text, color, .. } = line;
            let mut text = gfx::render::create_text(text, gres.get_font(font), font_size);
            gfx::render::set_text_fill_color(&mut text, *color);

            let txt_bounds = gfx::render::get_text_local_bounds(&text);
            max_row_width = max_row_width.max(txt_bounds.width);
            max_row_height = max_row_height.max(txt_bounds.height);

            texts.push((text, txt_bounds));
        }

        let position = self.position;
        let n_texts_f = texts.len() as f32;
        let tot_height = max_row_height * n_texts_f + row_spacing * (n_texts_f - 1.0);

        // Draw background
        gfx::render::fill_color_rect(
            window,
			self.config.background,
            Rect::new(
                position.x
                    + self
                        .horiz_align
                        .aligned_pos(0.0, 2.0 * pad_x + max_row_width),
                position.y + self.vert_align.aligned_pos(0.0, 2.0 * pad_y + tot_height),
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
                    self.horiz_align.aligned_pos(pad_x, max_row_width),
                    self.vert_align.aligned_pos(pad_y, tot_height)
                        + (i as f32) * (max_row_height + row_spacing),
                );
            let rect = Rect::new(pos.x, pos.y, bg_fill_ratio * max_row_width, max_row_height);
            gfx::render::fill_color_rect(
                window,
				bg_col,
                rect,
            );
        }

        // Draw texts
        for (i, (text, bounds)) in texts.iter_mut().enumerate() {
            let pos = Vec2f::new(
                self.horiz_align.aligned_pos(pad_x, bounds.width),
                self.vert_align.aligned_pos(pad_y, tot_height)
                    + (i as f32) * (max_row_height + row_spacing),
            );
            gfx::render::render_text(window, text, position + pos);
        }
    }
}
