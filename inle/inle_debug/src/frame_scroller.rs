use super::log::Debug_Log;
use inle_common::colors;
use inle_common::paint_props::Paint_Properties;
use inle_gfx::render;
use inle_gfx::render_window::Render_Window_Handle;
use inle_input::events::Input_Raw_Event;
use inle_input::input_state::Input_State;
use inle_input::keyboard::Key;
use inle_input::mouse::{self, Mouse_Button};
use inle_math::rect;
use inle_math::vector::{Vec2f, Vec2i, Vec2u};
use inle_resources::gfx::{Font_Handle, Gfx_Resources};
use std::time::Duration;

#[derive(Copy, Clone, PartialEq, Eq)]
enum Row {
    Seconds,
    Frames,
}

#[derive(Default)]
pub struct Debug_Frame_Scroller_Config {
    pub font: Font_Handle,
    pub font_size: u16,
}

#[derive(Default)]
pub struct Debug_Frame_Scroller {
    pub cfg: Debug_Frame_Scroller_Config,
    pub pos: Vec2u,
    pub size: Vec2u,
    /// How many frames are currently filled, in terms of n_frames * cur_second + cur_frame
    pub tot_scroller_filled_frames: u32,
    /// The current frame according to Debug_Log.
    real_cur_frame: u64,
    /// The currently latest filled frame in the Frame row.
    /// Note: 'cur_frame' is not an absolute value, but it always goes from 0 to n_frames - 1
    /// (so it does not map directly to Debug_Log's 'cur_frame', but it must be shifted)
    pub cur_frame: u16,
    /// The currently latest filled second in the Seconds row.
    /// Like cur_frame, it belongs to [0, n_seconds)
    pub cur_second: u16,
    /// The number of subdivisions of the 'Frame' row.
    pub n_frames: u16,
    /// The number of subdivisions of the 'Seconds' row.
    pub n_seconds: u16,
    /// How many subdivs in the 'Frame' row are currently filled.
    pub n_filled_frames: u16,
    /// How many subdivs in the 'Seconds' row are currently filled.
    pub n_filled_seconds: u16,
    pub manually_selected: bool,
    hovered: Option<(Row, u16)>,
}

struct Row_Props {
    pub y: f32,
    pub height: f32,
    pub subdivs: u16,
    pub filled: u16,
    pub show_labels: bool,
}

const DRAW_COLOR_HIGH_THRESHOLD_MS: u64 = 70;

impl Debug_Frame_Scroller {
    pub fn update(
        &mut self,
        window: &Render_Window_Handle,
        log: &Debug_Log,
        input_state: &Input_State,
    ) {
        trace!("frame_scroller::update");

        if !self.manually_selected {
            self.update_frame(log);
        }

        let mpos = mouse::mouse_pos_in_window(window, &input_state.raw.mouse_state);
        self.hovered = None;
        self.check_hovered_row(Row::Seconds, mpos);
        if self.hovered.is_none() {
            self.check_hovered_row(Row::Frames, mpos);
        }
    }

    fn update_frame(&mut self, log: &Debug_Log) {
        self.cur_frame = ((log.hist_len - 1) % self.n_frames as u32) as u16;
        self.n_filled_frames = self.cur_frame + 1;

        self.n_filled_seconds =
            ((log.hist_len - 1) / self.n_frames as u32).min(self.n_seconds as u32 - 1) as u16 + 1;
        self.cur_second = self.n_filled_seconds - 1;

        self.tot_scroller_filled_frames =
            self.cur_second as u32 * self.n_frames as u32 + self.cur_frame as u32 + 1;
        self.real_cur_frame = log.cur_frame;
    }

    pub fn handle_events(&mut self, events: &[Input_Raw_Event]) {
        fn calc_filled_frames(this: &Debug_Frame_Scroller) -> u16 {
            (this.tot_scroller_filled_frames - (this.cur_second as u32 * this.n_frames as u32))
                .min(this.n_frames as u32) as _
        }

        for event in events {
            match event {
                Input_Raw_Event::Mouse_Button_Pressed {
                    button: Mouse_Button::Left,
                } if self.hovered.is_some() => {
                    match self.hovered {
                        Some((Row::Frames, i)) => self.cur_frame = i,
                        Some((Row::Seconds, i)) => {
                            self.cur_second = i;
                            self.n_filled_frames = calc_filled_frames(self);
                        }
                        _ => unreachable!(),
                    }
                    self.manually_selected = true;
                }
                Input_Raw_Event::Mouse_Button_Pressed {
                    button: Mouse_Button::Right,
                } => {
                    self.manually_selected = false;
                }
                // @Incomplete: make this button configurable
                Input_Raw_Event::Key_Pressed { code: Key::Period } => {
                    if self.manually_selected
                        && self.cur_second as u32 * self.n_frames as u32 + (self.cur_frame as u32)
                            < self.tot_scroller_filled_frames
                    {
                        if self.cur_frame < self.n_filled_frames - 1 {
                            self.cur_frame += 1;
                        } else if self.cur_second < self.n_filled_seconds - 1 {
                            self.cur_frame = 0;
                            self.cur_second += 1;
                            self.n_filled_frames = calc_filled_frames(self);
                        }
                    }
                }
                // @Incomplete: make this button configurable
                Input_Raw_Event::Key_Pressed { code: Key::Comma } => {
                    if self.manually_selected {
                        if self.cur_frame > 0 {
                            self.cur_frame -= 1;
                        } else if self.cur_second > 0 {
                            self.n_filled_frames = self.n_frames;
                            self.cur_frame = self.n_filled_frames - 1;
                            self.cur_second -= 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn get_row_props(&self, row: Row) -> Row_Props {
        match row {
            Row::Seconds => Row_Props {
                y: self.pos.y as f32 + 1.,
                height: self.size.y as f32 * 0.5 - 2.,
                subdivs: self.n_seconds,
                filled: self.n_filled_seconds,
                show_labels: true,
            },
            Row::Frames => Row_Props {
                y: self.pos.y as f32 + self.size.y as f32 * 0.5,
                height: self.size.y as f32 * 0.5 - 2.,
                subdivs: self.n_frames,
                filled: self.n_filled_frames,
                show_labels: false,
            },
        }
    }

    fn check_hovered_row(&mut self, row: Row, mpos: Vec2i) {
        trace!("frame_scroller::check_hovered_row");

        let Row_Props {
            y,
            height,
            subdivs,
            filled,
            ..
        } = self.get_row_props(row);
        if filled > 0 {
            let subdiv_w = self.size.x as f32 / subdivs as f32 - 1.;
            for i in 0..filled {
                let subdiv_rect = rect::Rectf::new(
                    self.pos.x as f32 + i as f32 * (1. + subdiv_w),
                    y,
                    subdiv_w,
                    height,
                );
                let hovered = subdiv_rect.contains(mpos);
                if hovered {
                    debug_assert!(self.hovered.is_none());
                    self.hovered = Some((row, i as u16));
                }
            }
        }
    }

    pub fn draw(
        &self,
        window: &mut Render_Window_Handle,
        gres: &mut Gfx_Resources,
        debug_log: &Debug_Log,
    ) {
        trace!("frame_scroller::draw");

        let mut vbuf = render::start_draw_quads_temp(window, (self.n_frames + self.n_seconds) as _);

        self.draw_row(window, &mut vbuf, gres, Row::Seconds, debug_log);
        self.draw_row(window, &mut vbuf, gres, Row::Frames, debug_log);

        render::render_vbuf(window, &vbuf, &inle_math::transform::Transform2D::default());
    }

    fn draw_row(
        &self,
        window: &mut Render_Window_Handle,
        vbuf: &mut render::Vertex_Buffer_Quads,
        gres: &Gfx_Resources,
        row: Row,
        debug_log: &Debug_Log,
    ) {
        let Row_Props {
            y,
            height,
            subdivs,
            filled,
            show_labels,
        } = self.get_row_props(row);
        let cur = if row == Row::Frames {
            self.cur_frame
        } else {
            self.cur_second
        };

        let row_r = rect::Rectf::new(self.pos.x as _, y, self.size.x as _, height);
        let row_hovered = if let Some((r, _)) = self.hovered {
            r == row
        } else {
            false
        };
        {
            // Draw outline
            let paint_props = Paint_Properties {
                color: colors::TRANSPARENT,
                border_thick: 1.0,
                border_color: colors::rgba(200, 200, 200, if row_hovered { 250 } else { 0 }),
                ..Default::default()
            };
            render::render_rect(window, row_r, paint_props);
        }

        if subdivs == 0 {
            return;
        }

        let outline_col = if row_hovered || self.manually_selected {
            200
        } else {
            60
        };
        let subdiv_w = self.size.x as f32 / subdivs as f32 - 1.;

        for i in 0..subdivs {
            let subdiv_rect = rect::Rectf::new(
                self.pos.x as f32 + i as f32 * (1. + subdiv_w),
                y,
                subdiv_w,
                height,
            );
            let hovered = self.hovered == Some((row, i));

            let rgb = if row == Row::Seconds {
                self.calc_slot_color_seconds(debug_log, i)
            } else {
                self.calc_slot_color_frame(debug_log, i)
            };
            let color = if i as u16 != cur {
                let alpha = if hovered {
                    220
                } else if i < filled {
                    if self.manually_selected {
                        180
                    } else if row_hovered {
                        120
                    } else {
                        70
                    }
                } else {
                    20
                };
                colors::rgba(rgb.r, rgb.g, rgb.b, alpha)
            } else {
                colors::rgba(40, 100, 200, 240)
            };
            let paint_props = Paint_Properties {
                color,
                border_thick: 1.0,
                border_color: colors::rgba(outline_col, outline_col, outline_col, color.a),
                ..Default::default()
            };
            render::add_quad(
                vbuf,
                &render::new_vertex(subdiv_rect.pos_min(), paint_props.color, v2!(0., 0.)),
                &render::new_vertex(
                    subdiv_rect.pos_min() + v2!(subdiv_rect.width, 0.),
                    paint_props.color,
                    v2!(0., 0.),
                ),
                &render::new_vertex(subdiv_rect.pos_max(), paint_props.color, v2!(0., 0.)),
                &render::new_vertex(
                    subdiv_rect.pos_min() + v2!(0., subdiv_rect.height),
                    paint_props.color,
                    v2!(0., 0.),
                ),
            );
        }

        if show_labels {
            let font = gres.get_font(self.cfg.font);
            let text_col = if row_hovered || self.manually_selected {
                colors::WHITE
            } else {
                colors::rgba(120, 120, 120, 180)
            };
            for i in 0..filled {
                let x = self.pos.x as f32 + i as f32 * (1. + subdiv_w);
                // The very_first_frame is initially 1, but it can change if the game is paused and resumed
                // (in which case the debug log will drop old history and restart from a later frame).
                // It can also change simply due to the scroller filling up.
                let very_first_frame = self.real_cur_frame - self.tot_scroller_filled_frames as u64;
                let row_first_frame = (self.n_frames as u64 * i as u64) + very_first_frame;
                let mut text = render::create_text(
                    &(row_first_frame + 1).to_string(),
                    font,
                    self.cfg.font_size,
                );
                render::render_text(window, &mut text, text_col, Vec2f::new(x, y));
            }
        }
    }

    fn calc_slot_color_seconds(&self, debug_log: &Debug_Log, slot_idx: u16) -> colors::Color {
        let very_first_frame = self.real_cur_frame - self.tot_scroller_filled_frames as u64;
        let subdiv_first_frame = very_first_frame
            + if slot_idx > 0 {
                (slot_idx - 1) * self.n_frames
            } else {
                0
            } as u64;
        let subdiv_last_frame = subdiv_first_frame
            + if slot_idx == self.n_filled_seconds {
                self.n_filled_frames
            } else {
                self.n_frames
            } as u64;
        let tot_duration = (subdiv_first_frame..=subdiv_last_frame)
            .map(|n_frame| {
                debug_log.get_frame(n_frame).map(|frame| {
                    frame
                        .traces
                        .get(0)
                        .map(|t| t.info.tot_duration())
                        .unwrap_or_else(Duration::default)
                })
            })
            .flatten()
            .sum();
        colors::lerp_col(
            colors::GREEN,
            colors::RED,
            inle_core::time::duration_ratio(
                &tot_duration,
                &Duration::from_millis(DRAW_COLOR_HIGH_THRESHOLD_MS * self.n_frames as u64),
            ),
        )
    }

    fn calc_slot_color_frame(&self, debug_log: &Debug_Log, frame_idx: u16) -> colors::Color {
        debug_log
            .get_frame(self.map_frame_index_to_real_frame(frame_idx))
            .map(|frame| {
                frame
                    .traces
                    .get(0)
                    .map(|t| t.info.tot_duration())
                    .map(|frame_time| {
                        colors::lerp_col(
                            colors::GREEN,
                            colors::RED,
                            inle_core::time::duration_ratio(
                                &frame_time,
                                &Duration::from_millis(DRAW_COLOR_HIGH_THRESHOLD_MS),
                            ),
                        )
                    })
            })
            .flatten()
            .unwrap_or_else(|| colors::rgb(100, 100, 100))
    }

    fn map_frame_index_to_real_frame(&self, idx: u16) -> u64 {
        let very_first_frame = self.real_cur_frame - self.tot_scroller_filled_frames as u64;
        very_first_frame + self.cur_second as u64 * self.n_frames as u64 + idx as u64 + 1
    }

    pub fn get_real_selected_frame(&self) -> u64 {
        self.map_frame_index_to_real_frame(self.cur_frame)
    }

    pub fn set_real_selected_frame(&mut self, real_frame: u64) {
        // @FIXME!

        self.cur_frame = (real_frame % self.n_frames as u64) as u16;
        //if self.cur_frame >= self.n_filled_frames {
        //    self.n_filled_frames = self.cur_frame + 1;
        //}

        self.cur_second = (real_frame / self.n_frames as u64) as u16;
        //if self.cur_second >= self.n_filled_seconds {
        //    self.n_filled_seconds = (self.cur_second + 1).min(self.n_seconds)
        //}

        //self.tot_scroller_filled_frames =
        //    self.cur_second as u32 * self.n_frames as u32 + self.cur_frame as u32 + 1;
    }
}
