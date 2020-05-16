use super::state::{Game_State, Game_State_Args, State_Transition};
use ecs_engine::common::rect::Rect;
use ecs_engine::common::vector::Vec2f;
use ecs_engine::gfx::window::{self, Window_Handle};
use ecs_engine::ui;
use std::time::Duration;

#[derive(Default)]
struct Menu_Button {
    pub id: ui::UI_Id,
    pub props: ui::Button_Props,
    pub text: &'static str,
    pub size: Vec2f,
    pub pos: Vec2f,
}

#[derive(Default)]
pub struct Pause_Menu_State {
    buttons: Vec<Menu_Button>,
}

impl Pause_Menu_State {
    fn create_buttons(window: &Window_Handle) -> Vec<Menu_Button> {
        let mut buttons = vec![];
        let (ww, wh) = window::get_window_target_size(window);
        let ww = ww as f32;
        let wh = wh as f32;
        let props = ui::Button_Props {
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
            props: props.clone(),
            pos: v2!(tgx, tgy + size.y + spacing),
            text: "Quit",
            size: v2!(200., 120.),
        });
        buttons
    }
}

impl Game_State for Pause_Menu_State {
    fn on_start(&mut self, args: &mut Game_State_Args) {
        self.buttons = Self::create_buttons(args.window);
        args.engine_state.time.paused = true;
    }

    fn on_end(&mut self, args: &mut Game_State_Args) {
        args.engine_state.time.paused = false;
    }

    fn update(
        &mut self,
        args: &mut Game_State_Args,
        _dt: &Duration,
        _real_dt: &Duration,
    ) -> State_Transition {
        let window = &mut args.window;
        let gres = &args.game_resources.gfx;
        let ui_ctx = &mut args.engine_state.systems.ui;
        let istate = &args.engine_state.input_state;

        let b = &self.buttons[0];
        let rect = Rect::new(b.pos.x, b.pos.y, b.size.x, b.size.y);
        // Resume game
        if ui::button(window, gres, istate, ui_ctx, b.id, b.text, rect, &b.props) {
            return State_Transition::Pop;
        }

        let b = &self.buttons[1];
        let rect = Rect::new(b.pos.x, b.pos.y, b.size.x, b.size.y);
        // Quit to menu
        if ui::button(window, gres, istate, ui_ctx, b.id, b.text, rect, &b.props) {
            return State_Transition::Flush_All_And_Replace(Box::new(
                super::main_menu_state::Main_Menu_State::default(),
            ));
        }

        let b = &self.buttons[2];
        let rect = Rect::new(b.pos.x, b.pos.y, b.size.x, b.size.y);
        // Quit game
        if ui::button(window, gres, istate, ui_ctx, b.id, b.text, rect, &b.props) {
            args.engine_state.should_close = true;
        }

        State_Transition::None
    }
}
