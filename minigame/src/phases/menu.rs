use super::Phase_Args;
use inle_gfx::sprites;
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

    sprites: Vec<sprites::Sprite>,
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
        let size = v2!(200., 100.);
        let tgx = (ww - size.x) * 0.5;
        let tgy = (wh - size.y) * 0.5 + 180.0;
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
        if self.sprites.is_empty() {
            let gs = args.game_state();
            let (win_w, win_h) = gs.app_config.target_win_size;
            let tex_rect = inle_math::rect::Rect::new(0, 0, win_w as _, win_h as _);

            let mut res = args.game_res_mut();
            let gres = &mut res.gfx;
            let env = &gs.env;

            let tex_path = inle_resources::gfx::tex_path(env, "menu/main_menu_background.png");
            let mut sprite = sprites::Sprite::from_tex_path(gres, &tex_path);
            sprite.z_index = -1;
            self.sprites.push(sprite);

            let tex_path = inle_resources::gfx::tex_path(env, "menu/main_menu_mountains.png");
            let mut sprite = sprites::Sprite::from_tex_path(gres, &tex_path);
            sprite.rect = tex_rect;
            self.sprites.push(sprite);

            let tex_path = inle_resources::gfx::tex_path(env, "menu/game_logo.png");
            let mut sprite = sprites::Sprite::from_tex_path(gres, &tex_path);
            sprite.transform.translate(0., -200.);
            self.sprites.push(sprite);

            let tex_path = inle_resources::gfx::tex_path(env, "game/sun.png");
            let mut sprite = sprites::Sprite::from_tex_path(gres, &tex_path);
            self.sprites.push(sprite);
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

        for sprite in &self.sprites {
            inle_gfx::sprites::render_sprite(
                &mut gs.window,
                &mut gs.batches,
                sprite,
            );
        }

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
