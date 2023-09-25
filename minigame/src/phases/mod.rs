use std::cell::RefCell;
use std::rc::Rc;

//pub mod in_game_state;
//pub mod pause_menu_state;
//pub mod persistent;
pub mod menu;

pub struct Phase_Args<'a> {
    pub window: Rc<RefCell<inle_gfx::render_window::Render_Window_Handle>>,
    pub ui: Rc<RefCell<inle_ui::Ui_Context>>,
    pub input: Rc<RefCell<inle_input::input_state::Input_State>>,
    pub time: Rc<RefCell<inle_core::time::Time>>,
    pub game_res: &'a crate::Game_Resources,
}
