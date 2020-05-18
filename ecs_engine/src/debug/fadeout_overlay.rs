use super::element::Debug_Element;
use crate::alloc::temp;
use crate::common::colors::{self, Color};
use crate::common::rect::Rect;
use crate::common::vector::Vec2f;
use crate::core;
use crate::gfx;
use crate::gfx::align::Align;
use crate::gfx::window::Window_Handle;
use crate::input::input_state::Input_State;
use crate::resources::gfx::{Font_Handle, Gfx_Resources};
use std::collections::VecDeque;
use std::time::Duration;

struct Fadeout_Text {
    pub text: String,
    pub color: Color,
    pub time: Duration,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Fadeout_Debug_Overlay_Config {
    pub row_spacing: f32,
    pub font_size: u16,
    pub pad_x: f32,
    pub pad_y: f32,
    pub background: Color,
    pub max_rows: usize,
    pub fadeout_time: Duration,
    pub horiz_align: Align,
    pub vert_align: Align,
    pub font: Font_Handle,
}

#[derive(Default)]
pub struct Fadeout_Debug_Overlay {
    fadeout_texts: VecDeque<Fadeout_Text>,

    pub config: Fadeout_Debug_Overlay_Config,
    pub position: Vec2f,
}

impl Debug_Element for Fadeout_Debug_Overlay {
    fn update(&mut self, dt: &Duration, _window: &Window_Handle, _input_state: &Input_State) {
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
    fn draw(
        &self,
        window: &mut Window_Handle,
        gres: &mut Gfx_Resources,
        frame_alloc: &mut temp::Temp_Allocator,
    ) {
        trace!("fadeout_overlay::draw");

        if self.fadeout_texts.is_empty() {
            return;
        }

        let Fadeout_Debug_Overlay_Config {
            font,
            font_size,
            pad_x,
            pad_y,
            row_spacing,
            fadeout_time,
            horiz_align,
            vert_align,
            ..
        } = self.config;

        let mut texts = temp::excl_temp_array(frame_alloc);
        let mut max_row_height = 0f32;
        let mut max_row_width = 0f32;

        let font = gres.get_font(font);
        for line in self.fadeout_texts.iter() {
            let Fadeout_Text { text, color, time } = line;

            let d = core::time::duration_ratio(&time, &fadeout_time);
            let alpha = 255 - (d * d * 255.0f32) as u8;
            let text = gfx::render::create_text(text, font, font_size);
            let color = Color { a: alpha, ..*color };

            let txt_size = gfx::render::get_text_size(&text);
            max_row_width = max_row_width.max(txt_size.x);
            max_row_height = max_row_height.max(txt_size.y);

            texts.push((text, color, txt_size));
        }

        let position = self.position;
        let n_texts_f = texts.len() as f32;
        let tot_height = max_row_height * n_texts_f + row_spacing * (n_texts_f - 1.0);

        // Draw background
        gfx::render::render_rect(
            window,
            Rect::new(
                position.x + horiz_align.aligned_pos(0.0, 2.0 * pad_x + max_row_width),
                position.y + vert_align.aligned_pos(0.0, 2.0 * pad_y + tot_height),
                2.0 * pad_x + max_row_width,
                2.0 * pad_y + tot_height,
            ),
            self.config.background,
        );

        // Draw lines
        for (i, (text, color, text_size)) in texts.iter_mut().enumerate() {
            let pos = Vec2f::new(
                horiz_align.aligned_pos(pad_x, text_size.x),
                vert_align.aligned_pos(pad_y, tot_height)
                    + (i as f32) * (max_row_height + row_spacing),
            );
            gfx::render::render_text(window, text, *color, position + pos);
        }
    }
}

impl Fadeout_Debug_Overlay {
    pub fn new(config: Fadeout_Debug_Overlay_Config) -> Self {
        Self {
            config,
            ..Default::default()
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
