use super::drawing::Draw_Command;
use inle_core::env::Env_Info;
use inle_resources::gfx::{font_path, Font_Handle, Gfx_Resources};
use std::collections::VecDeque;

// Probably @Incomplete: we may want stuff like parent information here.
pub type UI_Id = u32;

#[derive(Default)]
pub struct UI_Context {
    hot: UI_Id,
    active: UI_Id,
    pub font: Font_Handle,

    pub(super) draw_cmd_queue: VecDeque<Draw_Command>,
}

pub(super) const UI_ID_INVALID: UI_Id = 0;

pub(super) fn add_draw_commands<T>(ui: &mut UI_Context, commands: T)
where
    T: std::iter::IntoIterator<Item = Draw_Command>,
{
    ui.draw_cmd_queue.extend(commands.into_iter());
}

#[inline]
pub(super) fn set_hot(ui: &mut UI_Context, id: UI_Id) {
    if ui.active == UI_ID_INVALID {
        ui.hot = id;
    }
}

#[inline]
pub(super) fn set_nonhot(ui: &mut UI_Context, id: UI_Id) {
    if ui.hot == id {
        ui.hot = UI_ID_INVALID;
    }
}

#[inline]
pub(super) fn is_hot(ui: &UI_Context, id: UI_Id) -> bool {
    ui.hot == id
}

#[inline]
pub(super) fn set_active(ui: &mut UI_Context, id: UI_Id) {
    ui.active = id;
}

#[inline]
pub(super) fn set_inactive(ui: &mut UI_Context, id: UI_Id) {
    debug_assert!(is_active(ui, id));
    ui.active = UI_ID_INVALID;
}

#[inline]
pub(super) fn is_active(ui: &UI_Context, id: UI_Id) -> bool {
    ui.active == id
}

pub fn init_ui(ui: &mut UI_Context, gres: &mut Gfx_Resources, env: &Env_Info) {
    const FONT_NAME: &str = "Hack-Regular.ttf";

    ui.font = gres.load_font(&font_path(env, FONT_NAME));
}
