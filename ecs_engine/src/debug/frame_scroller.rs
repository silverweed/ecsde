use super::log::Debug_Log;
use crate::common::colors;
use crate::common::rect;
use crate::common::vector::{Vec2f, Vec2i, Vec2u};
use crate::gfx::paint_props::Paint_Properties;
use crate::gfx::render;
use crate::gfx::window;
use crate::input::input_system::Input_Raw_Event;
use crate::resources::gfx::{Font_Handle, Gfx_Resources};

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
    pub tot_scroller_frames: u32, // in terms of n_frames * cur_second + cur_frame
    real_cur_frame: u64,
    // Note: 'cur_frame' is not an absolute value, but it always goes from 0 to n_frames - 1
    // (so it does not map directly to Debug_Log's 'cur_frame', but it must be shifted)
    pub cur_frame: u16,
    pub cur_second: u16,
    pub pos: Vec2u,
    pub size: Vec2u,
    pub n_frames: u16,
    pub n_seconds: u16,
    pub n_filled_frames: u16,
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

impl Debug_Frame_Scroller {
    pub fn update(&mut self, window: &window::Window_Handle, log: &Debug_Log) {
        self.update_frame(log);

        let mpos = window::mouse_pos_in_window(window);
        self.hovered = None;
        self.check_hovered_row(Row::Seconds, mpos);
        if self.hovered.is_none() {
            self.check_hovered_row(Row::Frames, mpos);
        }
    }

    fn update_frame(&mut self, log: &Debug_Log) {
        // @Fixme: make the frame bar continue to scroll even when log.hist_len is maxed
        if !self.manually_selected {
            self.cur_frame = ((log.hist_len - 1) % self.n_frames as u32) as u16;
            self.n_filled_frames = self.cur_frame + 1;

            self.n_filled_seconds = ((log.hist_len - 1) / self.n_frames as u32)
                .min(self.n_seconds as u32 - 1) as u16
                + 1;
            self.cur_second = self.n_filled_seconds - 1;

            self.tot_scroller_frames =
                self.cur_second as u32 * self.n_frames as u32 + self.cur_frame as u32 + 1;
            self.real_cur_frame = log.cur_frame;
        }
    }

    pub fn handle_events(&mut self, events: &[Input_Raw_Event]) {
        fn calc_filled_frames(this: &Debug_Frame_Scroller) -> u16 {
            (this.tot_scroller_frames - (this.cur_second as u32 * this.n_frames as u32))
                .min(this.n_frames as u32) as _
        };

        #[cfg(feature = "use-sfml")]
        use sfml::window::Event;

        #[cfg(feature = "use-sfml")]
        for event in events {
            match event {
                Event::MouseButtonPressed {
                    button: sfml::window::mouse::Button::Left,
                    ..
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
                Event::MouseButtonPressed {
                    button: sfml::window::mouse::Button::Right,
                    ..
                } => {
                    self.manually_selected = false;
                }
                // @Incomplete: make this button configurable
                Event::KeyPressed {
                    code: sfml::window::Key::Period,
                    ..
                } => {
                    if self.manually_selected {
                        if self.cur_second as u32 * self.n_frames as u32 + (self.cur_frame as u32)
                            < self.tot_scroller_frames
                        {
                            if self.cur_frame < self.n_filled_frames - 1 {
                                self.cur_frame += 1;
                            } else if self.cur_second < self.n_filled_seconds - 1 {
                                self.cur_frame = 0;
                                self.cur_second += 1;
                                self.n_filled_frames = calc_filled_frames(self);
                            }
                        }
                    } else {
                        self.manually_selected = true;
                    }
                }
                // @Incomplete: make this button configurable
                Event::KeyPressed {
                    code: sfml::window::Key::Comma,
                    ..
                } => {
                    if self.manually_selected {
                        if self.cur_frame > 0 {
                            self.cur_frame -= 1;
                        } else if self.cur_second > 0 {
                            self.n_filled_frames = self.n_frames;
                            self.cur_frame = self.n_filled_frames - 1;
                            self.cur_second -= 1;
                        }
                    } else {
                        self.manually_selected = true;
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
                let hovered = rect::rect_contains(&subdiv_rect, mpos.into());
                if hovered {
                    debug_assert!(self.hovered.is_none());
                    self.hovered = Some((row, i as u16));
                }
            }
        }
    }

    pub fn draw(&self, window: &mut window::Window_Handle, gres: &mut Gfx_Resources) {
        trace!("frame_scroller::draw");

        self.draw_row(window, gres, Row::Seconds);
        self.draw_row(window, gres, Row::Frames);
    }

    fn draw_row(&self, window: &mut window::Window_Handle, gres: &Gfx_Resources, row: Row) {
        let Row_Props {
            y,
            height,
            subdivs,
            filled,
            show_labels,
        } = self.get_row_props(row);
        let mpos = window::mouse_pos_in_window(window);
        let cur = if row == Row::Frames {
            self.cur_frame
        } else {
            self.cur_second
        };

        let row_r = rect::Rectf::new(self.pos.x as _, y, self.size.x as _, height);
        let row_hovered = rect::rect_contains(&row_r, mpos.into());
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

        let (filled_col, outline_col) = if row_hovered || self.manually_selected {
            (70, 200)
        } else {
            (30, 60)
        };
        if subdivs > 0 {
            let subdiv_w = self.size.x as f32 / subdivs as f32 - 1.;
            for i in 0..subdivs {
                let subdiv_rect = rect::Rectf::new(
                    self.pos.x as f32 + i as f32 * (1. + subdiv_w),
                    y,
                    subdiv_w,
                    height,
                );
                let hovered = self.hovered == Some((row, i));
                let color = if i as u16 != cur {
                    let c = if hovered {
                        160
                    } else if i < filled {
                        filled_col
                    } else {
                        20
                    };
                    colors::rgba(
                        c,
                        c,
                        c,
                        if hovered {
                            250
                        } else if i < filled {
                            120
                        } else {
                            20
                        },
                    )
                } else {
                    colors::rgba(40, 100, 200, 240)
                };
                let paint_props = Paint_Properties {
                    color,
                    border_thick: 1.0,
                    border_color: colors::rgba(outline_col, outline_col, outline_col, color.a),
                    ..Default::default()
                };
                render::render_rect(window, subdiv_rect, paint_props);
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
                    let very_first_frame = self.real_cur_frame - self.tot_scroller_frames as u64;
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
    }

    pub fn get_real_selected_frame(&self) -> u64 {
        let very_first_frame = self.real_cur_frame - self.tot_scroller_frames as u64;
        let selected_scroller_frame =
            self.cur_second as u32 * self.n_frames as u32 + self.cur_frame as u32 + 1;
        very_first_frame + selected_scroller_frame as u64
    }
}