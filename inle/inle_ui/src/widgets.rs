use super::drawing::*;
use super::ui_context::*;
use inle_common::colors::{self, Color};
use inle_gfx::render_window::Render_Window_Handle;
use inle_input::input_state::Input_State;
use inle_input::mouse::{mouse_went_down, mouse_went_up, Mouse_Button};
use inle_math::rect::Rectf;
use inle_math::vector::Vec2f;
use inle_resources::gfx::{Font_Handle, Gfx_Resources};
use inle_win::window::mouse_pos_in_window;

pub struct By_Activeness<T> {
    pub normal: T,
    pub hot: T,
    pub active: T,
}

impl<T: Default> Default for By_Activeness<T> {
    fn default() -> Self {
        Self {
            normal: T::default(),
            hot: T::default(),
            active: T::default(),
        }
    }
}

impl<T: Copy> By_Activeness<T> {
    fn single_value(val: T) -> Self {
        Self {
            normal: val,
            hot: val,
            active: val,
        }
    }
}

impl<T: Clone> Clone for By_Activeness<T> {
    fn clone(&self) -> Self {
        Self {
            normal: self.normal.clone(),
            hot: self.hot.clone(),
            active: self.active.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Button_Props {
    pub bg_color: By_Activeness<Color>,
    pub text_color: By_Activeness<Color>,
    pub border_color: By_Activeness<Color>,
    pub border_thick: By_Activeness<f32>,
    pub font: Option<Font_Handle>, // if None, will use the default font
    pub font_size: u16,            // this is used even if font == None
    pub enabled: bool,
}

impl Default for Button_Props {
    fn default() -> Self {
        Button_Props {
            bg_color: By_Activeness {
                normal: colors::rgb(200, 200, 200),
                hot: colors::WHITE,
                active: colors::YELLOW,
            },
            text_color: By_Activeness::single_value(colors::BLACK),
            border_color: By_Activeness::single_value(colors::BLACK),
            border_thick: By_Activeness::single_value(1.),
            font: None,
            font_size: 12,
            enabled: true,
        }
    }
}

pub fn button(
    window: &mut Render_Window_Handle,
    gres: &Gfx_Resources,
    input_state: &Input_State,
    ui: &mut UI_Context,
    id: UI_Id,
    text: &str,
    rect: Rectf,
    props: &Button_Props,
) -> bool {
    assert_ne!(id, UI_ID_INVALID);

    let mut result = false;
    if props.enabled {
        if is_active(ui, id) {
            if mouse_went_up(&input_state.raw.mouse_state, Mouse_Button::Left) {
                if is_hot(ui, id) {
                    result = true;
                }
                set_inactive(ui, id);
            }
        } else if is_hot(ui, id)
            && mouse_went_down(&input_state.raw.mouse_state, Mouse_Button::Left)
        {
            set_active(ui, id);
        }

        let mpos = mouse_pos_in_window(window);
        if rect.contains(Vec2f::from(mpos)) {
            set_hot(ui, id);
        } else {
            set_nonhot(ui, id);
        }
    }

    // draw stuff
    let cmds = draw_button(
        gres,
        ui,
        text,
        rect,
        is_active(ui, id),
        is_hot(ui, id),
        props,
    );
    add_draw_commands(ui, cmds);

    result
}
