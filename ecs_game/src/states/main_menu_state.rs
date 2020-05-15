use super::state::{Game_State, Game_State_Args, State_Transition};
use ecs_engine::common::rect::Rect;
use ecs_engine::common::vector::{lerp_v, Vec2f};
use ecs_engine::gfx::window::{self, Window_Handle};
use ecs_engine::ui;
use std::time::Duration;

#[derive(Default)]
struct Menu_Button {
    pub id: ui::UI_Id,
    pub props: ui::Button_Props,
    pub text: &'static str,
    pub size: Vec2f,
    pub start_pos: Vec2f,
    pub target_pos: Vec2f,
    pub ease_t: f32,
}

#[derive(Default)]
pub struct Main_Menu_State {
    buttons: Vec<Menu_Button>,
}

impl Main_Menu_State {
    fn create_buttons(window: &Window_Handle) -> Vec<Menu_Button> {
        let mut buttons = vec![];
        let (ww, wh) = window::get_window_target_size(window);
        let ww = ww as f32;
        let wh = wh as f32;
        buttons.push(Menu_Button {
            id: 1,
            props: ui::Button_Props::default(),
            start_pos: v2!(ww * 0.5, 0.),
            target_pos: v2!(ww * 0.5, wh * 0.5),
            text: "Start Game",
            size: v2!(200., 120.),
            ease_t: 0.,
        });
        buttons.push(Menu_Button {
            id: 2,
            props: ui::Button_Props::default(),
            start_pos: v2!(ww * 0.5, 0.),
            target_pos: v2!(ww * 0.5, wh * 0.5 + 100.),
            text: "Quit",
            size: v2!(200., 120.),
            ease_t: 0.,
        });
        buttons
    }
}

impl Game_State for Main_Menu_State {
    fn on_start(&mut self, args: &mut Game_State_Args) {
        self.buttons = Self::create_buttons(args.window);
    }

    fn update(
        &mut self,
        args: &mut Game_State_Args,
        dt: &Duration,
        _real_dt: &Duration,
    ) -> State_Transition {
        for button in &mut self.buttons {
            button.ease_t = (button.ease_t + dt.as_secs_f32()).min(1.);
        }

        let window = &mut args.window;
        let gres = &args.game_resources.gfx;
        let ui_ctx = &mut args.engine_state.systems.ui;
        let istate = &args.engine_state.input_state;

        let b = &self.buttons[0];
        let pos = lerp_v(b.start_pos, b.target_pos, b.ease_t);
        let rect = Rect::new(pos.x, pos.y, b.size.x, b.size.y);
        // Start game
        if ui::button(window, gres, istate, ui_ctx, b.id, b.text, rect, &b.props) {}

        let b = &self.buttons[1];
        let pos = lerp_v(b.start_pos, b.target_pos, b.ease_t);
        let rect = Rect::new(pos.x, pos.y, b.size.x, b.size.y);
        // Quit game
        if ui::button(window, gres, istate, ui_ctx, b.id, b.text, rect, &b.props) {
            args.engine_state.should_close = true;
        }

        State_Transition::None
    }
}
