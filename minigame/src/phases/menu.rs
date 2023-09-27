use super::Phase_Args;
use inle_app::phases::{Game_Phase, Phase_Transition};
use inle_common::stringid::{self, String_Id};
use inle_gfx::render_window::Render_Window_Handle;
use inle_math::rect::Rect;
use inle_math::vector::{lerp_v, Vec2f};
use inle_resources::gfx::Texture_Handle;
use inle_win::window;
use std::ops::DerefMut;
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
pub struct Main_Menu {
    buttons: Vec<Menu_Button>,

    tex_bg: Texture_Handle,
}

impl Main_Menu {
    pub const PHASE_ID: String_Id = stringid::const_sid_from_str("menu");

    pub fn new(window: &mut Render_Window_Handle) -> Self {
        Self {
            buttons: Self::create_buttons(window),
            ..Default::default()
        }
    }

    fn create_buttons(window: &Render_Window_Handle) -> Vec<Menu_Button> {
        let mut buttons = vec![];
        let (ww, wh) = window::get_window_target_size(window);
        let (ww, wh) = (ww as f32, wh as f32);
        let props = inle_ui::Button_Props {
            font_size: 24,
            ..Default::default()
        };
        let ease_duration = Duration::from_millis(300);
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

impl Game_Phase for Main_Menu {
    type Args = Phase_Args;

    fn on_start(&mut self, args: &mut Self::Args) {
        if self.tex_bg.is_none() {
            let mut res = args.game_res_mut();
            let env = &args.game_state().env;

            let bg_tex = inle_resources::gfx::tex_path(env, "menu/main_menu_background.png");
            self.tex_bg = res.gfx.load_texture(&bg_tex);
        }
    }

    fn update(&mut self, args: &mut Self::Args) -> Phase_Transition {
        let mut game_state = args.game_state_mut();
        let gs = game_state.deref_mut();
        let game_res = args.game_res();

        let dt = gs.time.dt();

        for button in &mut self.buttons {
            button.ease_t += dt.as_secs_f32();
        }

        let gres = &game_res.gfx;

        let (win_w, win_h) = gs.app_config.target_win_size;
        let material = inle_gfx::material::Material::with_texture(self.tex_bg);
        let tex_rect = inle_math::rect::Rect::new(0, 0, win_w as _, win_h as _);
        inle_gfx::render::render_texture_ws(
            &mut gs.window,
            &mut gs.batches,
            &material,
            &tex_rect,
            inle_common::colors::WHITE,
            &inle_math::transform::Transform2D::default(),
            0,
        );

        let b = &self.buttons[0];
        let pos = lerp_v(
            b.start_pos,
            b.target_pos,
            (b.ease_t / b.ease_duration.as_secs_f32()).min(1.0),
        );
        let rect = Rect::new(pos.x, pos.y, b.size.x, b.size.y);
        // Start game
        if inle_ui::button(
            &mut gs.window,
            gres,
            &gs.input,
            &mut gs.ui,
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
            &mut gs.window,
            gres,
            &gs.input,
            &mut gs.ui,
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
