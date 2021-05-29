use super::element::{Debug_Element, Draw_Args, Update_Args, Update_Res};
use inle_alloc::temp;
use inle_cfg::Cfg_Var;
use inle_common::colors::{self, Color};
use inle_common::vis_align::Align;
use inle_math::rect::Rect;
use inle_math::vector::Vec2f;
use inle_resources::gfx::Font_Handle;
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::time::Duration;

struct Fadeout_Text {
    pub text: String,
    pub color: Color,
    pub time: Duration,
}

#[derive(Clone, Default, Debug)]
pub struct Fadeout_Debug_Overlay_Config {
    pub ui_scale: Cfg_Var<f32>,
    pub row_spacing: Cfg_Var<f32>,
    pub font_size: Cfg_Var<u32>,
    pub pad_x: Cfg_Var<f32>,
    pub pad_y: Cfg_Var<f32>,
    pub background: Cfg_Var<u32>,   // Color
    pub fadeout_time: Cfg_Var<f32>, // seconds

    pub font: Font_Handle,
    pub horiz_align: Align,
    pub vert_align: Align,
    pub max_rows: usize,
}

#[derive(Default)]
pub struct Fadeout_Debug_Overlay {
    fadeout_texts: VecDeque<Fadeout_Text>,

    pub cfg: Fadeout_Debug_Overlay_Config,
    pub position: Vec2f,
}

impl Debug_Element for Fadeout_Debug_Overlay {
    fn update(&mut self, Update_Args { dt, config, .. }: Update_Args) -> Update_Res {
        trace!("debug::fadeout_overlay::update");

        let fadeout_time = Duration::from_secs_f32(self.cfg.fadeout_time.read(config));
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

        Update_Res::Stay_Enabled
    }

    // @Refactor: this is mostly @Cutnpaste from overlay.rs
    fn draw(
        &self,
        Draw_Args {
            window,
            gres,
            frame_alloc,
            config,
            ..
        }: Draw_Args,
    ) {
        trace!("debug::fadeout_overlay::draw");

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
            background,
            ui_scale,
            ..
        } = self.cfg;

        let ui_scale = ui_scale.read(config);
        let font_size = u16::try_from((font_size.read(config) as f32 * ui_scale) as u32).unwrap();
        // @FIXME: why are we crashing here when hotloading?
        for (k, v) in config.get_all_pairs() {
            println!("{} => {:?}", k, v);
        }
        let pad_x = pad_x.read(config) * ui_scale;
        let pad_y = pad_y.read(config) * ui_scale;
        let row_spacing = row_spacing.read(config) * ui_scale;
        let fadeout_time = Duration::from_secs_f32(fadeout_time.read(config));
        let background = colors::color_from_hex(background.read(config));

        let mut texts = temp::excl_temp_array(frame_alloc);
        let mut max_row_height = 0f32;
        let mut max_row_width = 0f32;

        let font = gres.get_font(font);
        for line in self.fadeout_texts.iter() {
            let Fadeout_Text { text, color, time } = line;

            let d = inle_core::time::duration_ratio(&time, &fadeout_time);
            let alpha = 255 - (d * d * 255.0f32) as u8;
            let text = inle_gfx::render::create_text(text, font, font_size);
            let color = Color { a: alpha, ..*color };

            let txt_size = inle_gfx::render::get_text_size(&text);
            max_row_width = max_row_width.max(txt_size.x);
            max_row_height = max_row_height.max(txt_size.y);

            texts.push((text, color, txt_size));
        }

        let position = self.position;
        let n_texts_f = texts.len() as f32;
        let tot_height = max_row_height * n_texts_f + row_spacing * (n_texts_f - 1.0);

        // Draw background
        inle_gfx::render::render_rect(
            window,
            Rect::new(
                position.x + horiz_align.aligned_pos(0.0, 2.0 * pad_x + max_row_width),
                position.y + vert_align.aligned_pos(0.0, 2.0 * pad_y + tot_height),
                2.0 * pad_x + max_row_width,
                2.0 * pad_y + tot_height,
            ),
            background,
        );

        // Draw lines
        for (i, (text, color, text_size)) in texts.iter_mut().enumerate() {
            let pos = Vec2f::new(
                horiz_align.aligned_pos(pad_x, text_size.x),
                vert_align.aligned_pos(pad_y, tot_height)
                    + (i as f32) * (max_row_height + row_spacing),
            );
            inle_gfx::render::render_text(window, text, *color, position + pos);
        }
    }
}

impl Fadeout_Debug_Overlay {
    pub fn new(cfg: &Fadeout_Debug_Overlay_Config) -> Self {
        Self {
            cfg: cfg.clone(),
            ..Default::default()
        }
    }

    pub fn add_line(&mut self, txt: &str) {
        self.add_line_color(txt, colors::rgb(255, 255, 255));
    }

    pub fn add_line_color(&mut self, txt: &str, color: Color) {
        if self.fadeout_texts.len() == self.cfg.max_rows {
            self.fadeout_texts.pop_front();
        }
        self.fadeout_texts.push_back(Fadeout_Text {
            text: String::from(txt),
            time: Duration::new(0, 0),
            color,
        });
    }
}
