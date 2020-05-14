use super::drawing::*;
use crate::common::rect::Rectf;
use crate::input::bindings::mouse::{mouse_went_up, mouse_went_down, Mouse_Button};
use crate::gfx::window::{Window_Handle, mouse_pos_in_window};
use crate::common::vector::Vec2f;
use crate::input::input_system::Input_State;

// in game code:
// for menus
//   for items
//      do items
// for popups
//   do popup

pub type UI_Id = u32; // TEMP

#[derive(Default)]
pub struct UI_Context {
    hot: UI_Id,
    active: UI_Id,
}

const UI_ID_INVALID: UI_Id = 0;

#[inline]
fn set_hot(ui: &mut UI_Context, id: UI_Id) {
    if ui.active == UI_ID_INVALID {
        ui.hot = id;
    }
}

#[inline]
fn is_hot(ui: &UI_Context, id: UI_Id) -> bool {
    ui.hot == id
}

#[inline]
fn set_active(ui: &mut UI_Context, id: UI_Id) {
    ui.active = id;
}

#[inline]
fn set_inactive(ui: &mut UI_Context, id: UI_Id) {
    debug_assert!(is_active(ui, id));
    ui.active = UI_ID_INVALID;
}

#[inline]
fn is_active(ui: &UI_Context, id: UI_Id) -> bool {
    ui.active == id
}

pub fn button(window: &mut Window_Handle, input_state: &Input_State, ui: &mut UI_Context, id: UI_Id, text: &str, rect: Rectf) -> bool {
    let mut result = false;
    if is_active(ui, id) {
        if mouse_went_up(&input_state.mouse_state, Mouse_Button::Left)  {
            if is_hot(ui, id) {
                result = true;
            }
            set_inactive(ui, id);
        }
    } else if is_hot(ui, id) {
        if mouse_went_down(&input_state.mouse_state, Mouse_Button::Left) {
            set_active(ui, id);
        }
    }

    let mpos = mouse_pos_in_window(window);
    if rect.contains(Vec2f::from(mpos)) {
        set_hot(ui, id);
    }

    // draw stuff
    draw_button(text, rect);

    result
}
