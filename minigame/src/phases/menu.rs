use super::Phase_Args;
use inle_app::phases::{Game_Phase, Phase_Transition};
use inle_gfx::render_window::Render_Window_Handle;
use inle_math::rect::Rect;
use inle_math::vector::{lerp_v, Vec2f};
use inle_win::window;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

#[derive(Default)]
struct Menu_Button {
    pub id: inle_ui::Ui_Id,
    pub props: inle_ui::Button_Props,
    pub text: &'static str,
    pub size: Vec2f,
    pub start_pos: Vec2f,
    pub target_pos: Vec2f,
    pub ease_t: f32,
    pub ease_duration: Duration,
}

#[derive(Default)]
pub struct Main_Menu<'s> {
    buttons: Vec<Menu_Button>,
    _pd: std::marker::PhantomData<&'s ()>,
}

impl Main_Menu<'_> {
    fn create_buttons(window: &Render_Window_Handle) -> Vec<Menu_Button> {
        let mut buttons = vec![];
        let (ww, wh) = window::get_window_target_size(window);
        let (ww, wh) = (ww as f32, wh as f32);
        let props = inle_ui::Button_Props {
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

impl<'s> Game_Phase for Main_Menu<'s> {
    type Args = Phase_Args<'s>;

    fn on_start(&mut self, args: &mut Self::Args) {
        self.buttons = Self::create_buttons(args.window.borrow_mut().deref_mut());
    }

    fn update(&mut self, args: &mut Self::Args) -> Phase_Transition {
        let dt = args.time.borrow().dt();

        for button in &mut self.buttons {
            button.ease_t += dt.as_secs_f32();
        }

        let mut window = args.window.borrow_mut();
        let gres = &args.game_res.gfx;
        let mut ui_ctx = args.ui.borrow_mut();
        let istate = args.input.borrow();

        let b = &self.buttons[0];
        let pos = lerp_v(
            b.start_pos,
            b.target_pos,
            (b.ease_t / b.ease_duration.as_secs_f32()).min(1.0),
        );
        let rect = Rect::new(pos.x, pos.y, b.size.x, b.size.y);
        // Start game
        if inle_ui::button(
            window.deref_mut(),
            gres,
            istate.deref(),
            ui_ctx.deref_mut(),
            b.id,
            b.text,
            rect,
            &b.props,
        ) {
            return Phase_Transition::None;
            //return Phase_Transition::Push(
            //    Box::new(super::in_game_state::In_Game_State::default()),
            //);
        }

        let b = &self.buttons[1];
        let pos = lerp_v(
            b.start_pos,
            b.target_pos,
            (b.ease_t / b.ease_duration.as_secs_f32()).min(1.0),
        );
        let rect = Rect::new(pos.x, pos.y, b.size.x, b.size.y);
        // Quit game
        if inle_ui::button(
            window.deref_mut(),
            gres,
            istate.deref(),
            ui_ctx.deref_mut(),
            b.id,
            b.text,
            rect,
            &b.props,
        ) {
            Phase_Transition::Quit_Game
        } else {
            Phase_Transition::None
        }
    }
}
