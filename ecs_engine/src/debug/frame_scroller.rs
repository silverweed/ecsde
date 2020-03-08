use crate::common::colors;
use crate::common::rect;
use crate::common::vector::Vec2u;
use crate::gfx::paint_props::Paint_Properties;
use crate::gfx::render;
use crate::gfx::window;

#[derive(Default)]
pub struct Debug_Frame_Scroller {
    // If Some, the debug is at some specific frame, else at latest.
    // Note: 'cur_frame' is not an absolute value, but it always goes from 0 to n_frames - 1
    // (so it does not map directly to Debug_Log's 'cur_frame', but it must be shifted)
    pub cur_frame: Option<usize>,
    pub cur_second: Option<usize>,
    pub pos: Vec2u,
    pub size: Vec2u,
    pub n_frames: usize,
    pub n_seconds: usize,
}

impl Debug_Frame_Scroller {
    pub fn update(&mut self, window: &window::Window_Handle) {
        //let mpos = window::mouse_pos_in_window(window);
        //let r = rect::Recti::new(
        //self.pos.x as _,
        //self.pos.y as _,
        //self.size.x as _,
        //self.size.y as _,
        //);
        //let mouse_inside = rect::rect_contains(&r, mpos);
    }

    pub fn draw(&self, window: &mut window::Window_Handle) {
        trace!("frame_scroller::draw");

        self.draw_bar_at(
            window,
            self.pos.y as f32 + 1.,
            self.size.y as f32 * 0.5 - 2.,
            self.n_seconds,
        );
        self.draw_bar_at(
            window,
            self.pos.y as f32 + self.size.y as f32 * 0.5,
            self.size.y as f32 * 0.5 - 2.,
            self.n_frames,
        );
    }

    fn draw_bar_at(&self, window: &mut window::Window_Handle, y: f32, height: f32, subdivs: usize) {
        let mpos = window::mouse_pos_in_window(window);
        if subdivs > 0 {
            let subdiv_w = self.size.x as f32 / subdivs as f32 - 1.;
            let cur_frame = self.cur_frame.unwrap_or(self.n_frames - 1);
            for i in 0..subdivs {
                let subdiv_rect = rect::Rectf::new(
                    self.pos.x as f32 + i as f32 * (1. + subdiv_w),
                    y,
                    subdiv_w,
                    height,
                );
                let hovered = rect::rect_contains(&subdiv_rect, mpos.into());
                let color = if i != cur_frame {
                    let c = if hovered { 160 } else { 20 };
                    colors::rgba(c, c, c, if hovered { 250 } else { 100 })
                } else {
                    colors::rgba(40, 100, 200, 240)
                };
                let paint_props = Paint_Properties {
                    color,
                    border_thick: 1.0,
                    border_color: colors::rgba(200, 200, 200, color.a),
                    ..Default::default()
                };
                render::fill_color_rect(window, paint_props, subdiv_rect);
            }
        }

        {
            // Draw outline
            let r = rect::Rectf::new(self.pos.x as _, y, self.size.x as _, height);
            let hovered = rect::rect_contains(&r, mpos.into());
            let paint_props = Paint_Properties {
                color: colors::TRANSPARENT,
                border_thick: 1.0,
                border_color: colors::rgba(200, 200, 200, if hovered { 250 } else { 0 }),
                ..Default::default()
            };
            render::fill_color_rect(window, paint_props, r);
        }
    }
}
