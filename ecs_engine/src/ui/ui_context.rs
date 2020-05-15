use crate::core::env::Env_Info;
use crate::resources::gfx::{font_path, Font_Handle, Gfx_Resources};

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
    pub font: Font_Handle,
}

pub(super) const UI_ID_INVALID: UI_Id = 0;

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
