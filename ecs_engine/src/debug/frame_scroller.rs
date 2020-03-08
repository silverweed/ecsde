use crate::common::colors;
use crate::common::rect;
use crate::common::vector::{Vec2i, Vec2u};
use crate::gfx::paint_props::Paint_Properties;
use crate::gfx::render;
use crate::gfx::window;
use crate::input::input_system::Input_Raw_Event;

#[derive(Copy, Clone, PartialEq, Eq)]
enum Row {
    Seconds,
    Frames,
}

#[derive(Default)]
pub struct Debug_Frame_Scroller {
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

impl Debug_Frame_Scroller {
    pub fn update(&mut self, window: &window::Window_Handle) {
        let mpos = window::mouse_pos_in_window(window);

        self.hovered = None;
        self.check_hovered_row(Row::Seconds, mpos);
        if self.hovered.is_none() {
            self.check_hovered_row(Row::Frames, mpos);
        }
    }

    pub fn handle_events(&mut self, events: &[Input_Raw_Event]) {
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
                } if self.manually_selected => {
                    if self.cur_frame < self.n_frames {
                        self.cur_frame += 1;
                    } else if self.cur_second < self.n_seconds {
                        self.cur_frame = 0;
                        self.cur_second += 1;
                    }
                }
                // @Incomplete: make this button configurable
                Event::KeyPressed {
                    code: sfml::window::Key::Comma,
                    ..
                } if self.manually_selected => {
                    if self.cur_frame > 0 {
                        self.cur_frame -= 1;
                    } else if self.cur_second > 0 {
                        self.cur_frame = self.n_frames - 1;
                        self.cur_second -= 1;
                    }
                }
                _ => {}
            }
        }
    }

    fn get_row_y_height_subdivs_filled(&self, row: Row) -> (f32, f32, u16, u16) {
        match row {
            Row::Seconds => (
                self.pos.y as f32 + 1.,
                self.size.y as f32 * 0.5 - 2.,
                self.n_seconds,
                self.n_filled_seconds,
            ),
            Row::Frames => (
                self.pos.y as f32 + self.size.y as f32 * 0.5,
                self.size.y as f32 * 0.5 - 2.,
                self.n_frames,
                self.n_filled_frames,
            ),
        }
    }

    fn check_hovered_row(&mut self, row: Row, mpos: Vec2i) {
        let (y, height, subdivs, filled) = self.get_row_y_height_subdivs_filled(row);
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

    pub fn draw(&self, window: &mut window::Window_Handle) {
        trace!("frame_scroller::draw");

        self.draw_row(window, Row::Seconds);
        self.draw_row(window, Row::Frames);
    }

    fn draw_row(&self, window: &mut window::Window_Handle, row: Row) {
        let (y, height, subdivs, filled) = self.get_row_y_height_subdivs_filled(row);
        let mpos = window::mouse_pos_in_window(window);
        let cur = if row == Row::Frames {
            self.cur_frame
        } else {
            self.cur_second
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
                        70
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
