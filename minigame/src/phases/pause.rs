use inle_app::phases::{Game_Phase, Phase_Id, Phase_Transition};
use super::Phase_Args;
use inle_gfx::render_window::Render_Window_Handle;
use inle_input::input_state::{Action_Kind, Game_Action};
use inle_math::rect::Rect;
use inle_math::vector::Vec2f;
use inle_win::window;
use std::time::Duration;
use std::ops::DerefMut;

#[derive(Default)]
struct Menu_Button {
    pub id: inle_ui::Ui_Id,
    pub props: inle_ui::Button_Props,
    pub text: &'static str,
    pub size: Vec2f,
    pub pos: Vec2f,
}

#[derive(Default)]
pub struct Pause_Menu  {
    buttons: Vec<Menu_Button>,
}

impl Pause_Menu {
    pub const PHASE_ID: Phase_Id = Phase_Id::new("pause");

    fn create_buttons(window: &Render_Window_Handle) -> Vec<Menu_Button> {
        let mut buttons = vec![];
        let (ww, wh) = window::get_window_target_size(window);
        let ww = ww as f32;
        let wh = wh as f32;
        let props = inle_ui::Button_Props {
            font_size: 24,
            ..Default::default()
        };
        let size = v2!(200., 120.);
        let tgx = (ww - size.x) * 0.5;
        let tgy = (wh - size.y) * 0.5;
        let spacing = 5.;
        buttons.push(Menu_Button {
            id: 1,
            props: props.clone(),
            pos: v2!(tgx, tgy - size.y - spacing),
            text: "Resume Game",
            size: v2!(200., 120.),
        });
        buttons.push(Menu_Button {
            id: 2,
            props: props.clone(),
            pos: v2!(tgx, tgy),
            text: "Quit To Menu",
            size: v2!(200., 120.),
        });
        buttons.push(Menu_Button {
            id: 3,
            props,
            pos: v2!(tgx, tgy + size.y + spacing),
            text: "Quit",
            size: v2!(200., 120.),
        });
        buttons
    }
}

impl Game_Phase for Pause_Menu {
    type Args = Phase_Args;

    fn on_start(&mut self, args: &mut Phase_Args) {
        let mut game_state = args.game_state_mut();
        self.buttons = Self::create_buttons(&mut game_state.window);
        game_state.time.paused = true;
    }

    fn on_end(&mut self, args: &mut Phase_Args) {
        let mut game_state = args.game_state_mut();
        game_state.time.paused = false;
    }

    fn update(
        &mut self,
        args: &mut Phase_Args,
    ) -> Phase_Transition {

        let mut game_state = args.game_state_mut();
        let gs = game_state.deref_mut();
        let istate = &gs.input;

        for action in &istate.processed.game_actions {
            match action {
                (name, Action_Kind::Pressed) if *name == sid!("open_pause_menu") => {
                    return Phase_Transition::Pop;
                }
                _ => (),
            }
        }

        let game_res = args.game_res();
        let window = &mut gs.window;
        let gres = &game_res.gfx;
        let ui_ctx = &mut gs.ui;

        let b = &self.buttons[0];
        let rect = Rect::new(b.pos.x, b.pos.y, b.size.x, b.size.y);
        // Resume game
        if inle_ui::button(window, gres, istate, ui_ctx, b.id, b.text, rect, &b.props) {
            return Phase_Transition::Pop;
        }

        let b = &self.buttons[1];
        let rect = Rect::new(b.pos.x, b.pos.y, b.size.x, b.size.y);
        // Quit to menu
        if inle_ui::button(window, gres, istate, ui_ctx, b.id, b.text, rect, &b.props) {
            return Phase_Transition::Flush_All_And_Replace(super::Main_Menu::PHASE_ID);
        }

        let b = &self.buttons[2];
        let rect = Rect::new(b.pos.x, b.pos.y, b.size.x, b.size.y);
        // Quit game
        if inle_ui::button(window, gres, istate, ui_ctx, b.id, b.text, rect, &b.props) {
            return Phase_Transition::Quit_Game;
        }

        Phase_Transition::None
    }
}
