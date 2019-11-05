use crate::core;
use crate::core::common::colors::{self, Color};
use crate::core::common::rect::Rect;
use crate::core::common::vector::{to_framework_vec, Vec2f};
use crate::gfx;
use crate::gfx::align::Align;
use crate::gfx::window::Window_Handle;
use crate::resources::gfx::{Font_Handle, Gfx_Resources};
use std::collections::VecDeque;
use std::time::Duration;

#[cfg(feature = "use-sfml")]
use sfml::graphics::Text;
#[cfg(feature = "use-sfml")]
use sfml::graphics::Transformable;

struct Fadeout_Text {
    pub text: String,
    pub color: Color,
    pub time: Duration,
}

#[derive(Copy, Clone)]
pub struct Fadeout_Debug_Overlay_Config {
    pub row_spacing: f32,
    pub font_size: u16,
    pub pad_x: f32,
    pub pad_y: f32,
    pub background: Color,
    pub max_rows: usize,
    pub fadeout_time: Duration,
}

pub struct Fadeout_Debug_Overlay {
    font: Font_Handle,
    fadeout_texts: VecDeque<Fadeout_Text>,
    config: Fadeout_Debug_Overlay_Config,

    pub position: Vec2f,
    pub horiz_align: Align,
    pub vert_align: Align,
}

impl Fadeout_Debug_Overlay {
    pub fn new(config: Fadeout_Debug_Overlay_Config, font: Font_Handle) -> Fadeout_Debug_Overlay {
        Fadeout_Debug_Overlay {
            font,
            fadeout_texts: VecDeque::with_capacity(config.max_rows),
            config,
            position: Vec2f::new(0.0, 0.0),
            horiz_align: Align::Begin,
            vert_align: Align::Begin,
        }
    }

    pub fn update(&mut self, dt: &Duration) {
        let fadeout_time = self.config.fadeout_time;
        let mut n_drained = 0;
        for (i, text) in self.fadeout_texts.iter_mut().enumerate().rev() {
            text.time += *dt;
            if text.time >= fadeout_time {
                // All following texts must have a time >= fadeout_time, since they're sorted by insertion time.
                n_drained = i + 1;
                break;
            }
        }
        for _ in 0..n_drained {
            self.fadeout_texts.pop_front();
        }
    }

    // @Refactor: this is mostly @Cutnpaste from overlay.rs
    pub fn draw(&mut self, window: &mut Window_Handle, gres: &mut Gfx_Resources) {
        let font = self.font;
        let Fadeout_Debug_Overlay_Config {
            font_size,
            pad_x,
            pad_y,
            row_spacing,
            fadeout_time,
            ..
        } = self.config;

        let mut texts = vec![];
        let mut max_row_height = 0f32;
        let mut max_row_width = 0f32;

        for line in self.fadeout_texts.iter() {
            let Fadeout_Text { text, color, time } = line;

            let d = core::time::duration_ratio(&time, &fadeout_time);
            let alpha = 255 - (d * d * 255.0f32) as u8;
            let mut text = Text::new(&text, gres.get_font(font), font_size.into());
            text.set_fill_color(&colors::rgba(color.r, color.g, color.b, alpha));

            let txt_bounds = text.local_bounds();
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

        // Draw lines
        for (i, (text, bounds)) in texts.iter_mut().enumerate() {
            let pos = Vec2f::new(
                self.horiz_align.aligned_pos(pad_x, bounds.width),
                self.vert_align.aligned_pos(pad_y, tot_height)
                    + (i as f32) * (max_row_height + row_spacing),
            );
            text.set_position(to_framework_vec(position + pos));

            gfx::render::render_text(window, &text);
        }
    }

    pub fn add_line(&mut self, txt: &str) {
        self.add_line_color(txt, colors::rgb(255, 255, 255));
    }

    pub fn add_line_color(&mut self, txt: &str, color: Color) {
        if self.fadeout_texts.len() == self.config.max_rows {
            self.fadeout_texts.pop_front();
        }
        self.fadeout_texts.push_back(Fadeout_Text {
            text: String::from(txt),
            time: Duration::new(0, 0),
            color,
        });
    }
}
