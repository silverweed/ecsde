use super::state::{Game_State, Game_State_Args, State_Transition};
use ecs_engine::common::rect::Rect;
use ecs_engine::common::vector::{lerp_v, Vec2f};
use ecs_engine::gfx::render_window::Render_Window_Handle;
use ecs_engine::gfx::window;
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
    pub ease_duration: Duration,
}

#[derive(Default)]
pub struct Main_Menu_State {
    buttons: Vec<Menu_Button>,
}

impl Main_Menu_State {
    fn create_buttons(window: &Render_Window_Handle) -> Vec<Menu_Button> {
        let mut buttons = vec![];
        let (ww, wh) = window::get_window_target_size(window);
        let (ww, wh) = (ww as f32, wh as f32);
        let props = ui::Button_Props {
            font_size: 24,
            ..Default::default()
        };
        let ease_duration = Duration::from_millis(400);
        let size = v2!(200., 120.);
        let tgx = (ww - size.x) * 0.5;
        let tgy = (wh - size.y) * 0.5;
        let spacing = 5.;
        buttons.push(Menu_Button {
            id: 1,
            props: props.clone(),
            start_pos: v2!(tgx, 0.),
            target_pos: v2!(tgx, tgy),
            text: "Start Game",
            size,
            ease_t: 0.,
            ease_duration,
        });
        buttons.push(Menu_Button {
            id: 2,
            props,
            start_pos: v2!(tgx, 0.),
            target_pos: v2!(tgx, tgy + size.y + spacing),
            text: "Quit",
            size,
            ease_t: 0.,
            ease_duration,
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
            button.ease_t += dt.as_secs_f32();
        }

        let window = &mut args.window;
        let gres = &args.game_resources.gfx;
        let ui_ctx = &mut args.engine_state.systems.ui;
        let istate = &args.engine_state.input_state;

        let b = &self.buttons[0];
        let pos = lerp_v(
            b.start_pos,
            b.target_pos,
            (b.ease_t / b.ease_duration.as_secs_f32()).min(1.0),
        );
        let rect = Rect::new(pos.x, pos.y, b.size.x, b.size.y);
        // Start game
        if ui::button(window, gres, istate, ui_ctx, b.id, b.text, rect, &b.props) {
            return State_Transition::Push(
                Box::new(super::in_game_state::In_Game_State::default()),
            );
        }

        let b = &self.buttons[1];
        let pos = lerp_v(
            b.start_pos,
            b.target_pos,
            (b.ease_t / b.ease_duration.as_secs_f32()).min(1.0),
        );
        let rect = Rect::new(pos.x, pos.y, b.size.x, b.size.y);
        // Quit game
        if ui::button(window, gres, istate, ui_ctx, b.id, b.text, rect, &b.props) {
            args.engine_state.should_close = true;
        }

        State_Transition::None
    }
}
