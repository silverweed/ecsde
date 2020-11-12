use super::element::Debug_Element;
use inle_alloc::temp;
use inle_common::colors::{self, Color};
use inle_common::stringid::String_Id;
use inle_common::variant::Variant;
use inle_common::vis_align::Align;
use inle_gfx::render_window::Render_Window_Handle;
use inle_input::input_state::Input_State;
use inle_math::rect::Rect;
use inle_math::vector::Vec2f;
use inle_resources::gfx::{Font_Handle, Gfx_Resources};
use std::collections::HashMap;

pub struct Debug_Line {
    pub text: String,
    pub color: Color,
    // Contains (fill_color, horizontal_fill_ratio 0-1)
    pub bg_rect_fill: Option<(Color, f32)>,
    pub metadata: HashMap<String_Id, Variant>,
}

impl Debug_Line {
    pub fn with_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    pub fn with_bg_rect_fill(&mut self, color: Color, fill: f32) -> &mut Self {
        self.bg_rect_fill = Some((color, fill));
        self
    }

    pub fn with_metadata<T: Into<Variant>>(&mut self, key: String_Id, metadata: T) -> &mut Self {
        self.metadata.insert(key, metadata.into());
        self
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Debug_Overlay_Config {
    pub row_spacing: f32,
    pub font_size: u16,
    pub pad_x: f32,
    pub pad_y: f32,
    pub background: Color,
    pub font: Font_Handle,
    pub horiz_align: Align,
    pub vert_align: Align,
    pub hoverable: bool,
}

#[derive(Default, Clone)]
pub struct Hover_Data {
    pub hovered_line: Option<usize>,

    /// Note: this value represents the *index* of the selected line, therefore if a logic
    /// involving the content of that line needs to be carried on for multiple frames, and if the lines
    /// of this Overlay can change, that content should be cloned somewhere, as in the next frame
    /// this same index may refer to a totally different line!
    ///
    /// e.g.
    ///
    /// Frame 1
    ///       Line A
    ///     > Line B <  (selected)
    ///       Line C
    ///
    /// ... query the selected line and do some logic regarding Line B (like using it as a function
    /// name to query the debug tracer).
    ///
    /// Frame 2
    ///      Line B
    ///    > Line A < (the index didn't change, but the line did!)
    ///      Line C
    ///
    /// ... if the content of Line B was not saved, but rather the overlay is blindly indexed with
    /// the selected index, we will do a totally wrong logic!
    pub selected_line: Option<usize>,

    /// Whether the selection changed this frame or not
    pub just_selected: bool,
}

#[derive(Default)]
pub struct Debug_Overlay {
    pub lines: Vec<Debug_Line>,

    pub config: Debug_Overlay_Config,
    pub position: Vec2f,

    // Latest drawn row bounds
    max_row_bounds: std::cell::Cell<(f32, f32)>,

    pub hover_data: Hover_Data,
}

impl Debug_Element for Debug_Overlay {
    fn draw(
        &self,
        window: &mut Render_Window_Handle,
        gres: &mut Gfx_Resources,
        frame_alloc: &mut temp::Temp_Allocator,
    ) {
        trace!("debug::overlay::draw");

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

        let mut texts = temp::excl_temp_array(frame_alloc);
        let mut max_row_width = 0f32;
        let mut max_row_height = 0f32;

        let font = gres.get_font(font);
        for line in self.lines.iter() {
            let Debug_Line { text, color, .. } = line;
            let text = inle_gfx::render::create_text(text, font, font_size);

            let txt_size = inle_gfx::render::get_text_size(&text);
            max_row_width = max_row_width.max(txt_size.x);
            max_row_height = max_row_height.max(txt_size.y);

            texts.push((text, *color, txt_size));
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
            self.config.background,
        );

        self.max_row_bounds.set((max_row_width, max_row_height));

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
            inle_gfx::render::render_rect(window, rect, bg_col);
        }

        // Draw texts
        for (i, (text, color, text_size)) in texts.iter_mut().enumerate() {
            let pos = Vec2f::new(
                horiz_align.aligned_pos(pad_x, text_size.x),
                vert_align.aligned_pos(pad_y, tot_height)
                    + (i as f32) * (max_row_height + row_spacing),
            );
            // @Incomplete
            if let Some(line) = self.hover_data.hovered_line {
                if line == i {
                    *color = colors::WHITE;
                }
            }
            inle_gfx::render::render_text(window, text, *color, position + pos);
        }
    }

    fn update(
        &mut self,
        _dt: &std::time::Duration,
        window: &Render_Window_Handle,
        input_state: &Input_State,
    ) {
        use inle_input::mouse;
        use inle_win::window;

        trace!("debug::overlay::update");

        if !self.config.hoverable {
            return;
        }

        let (row_width, row_height) = self.max_row_bounds.get();
        // @Incomplete: this calculation is probably broken for alignments different from Align_Middle
        let Vec2f { x: sx, y: sy } = self.position + v2!(self.config.pad_x, self.config.pad_y)
            - v2!(
                row_width * 0.5,
                (row_height + self.config.row_spacing) * (self.lines.len() as f32) * 0.5
            );
        let mpos = Vec2f::from(window::mouse_pos_in_window(window));

        if mouse::is_mouse_btn_pressed(&input_state.raw.mouse_state, mouse::Mouse_Button::Left) {
            self.hover_data.selected_line = self.hover_data.hovered_line;
            self.hover_data.just_selected = true;
        } else {
            self.hover_data.just_selected = false;
        }

        self.hover_data.hovered_line = None;
        for i in 0..self.lines.len() {
            let line_rect = Rect::new(
                sx,
                sy + (i as f32) * (row_height + self.config.row_spacing),
                row_width,
                row_height,
            );
            if line_rect.contains(mpos) {
                self.hover_data.hovered_line = Some(i);
                break;
            }
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

    pub fn add_line(&mut self, line: &str) -> &mut Debug_Line {
        self.lines.push(Debug_Line {
            text: String::from(line),
            color: colors::WHITE,
            bg_rect_fill: None,
            metadata: HashMap::default(),
        });
        let len = self.lines.len();
        &mut self.lines[len - 1]
    }
}
