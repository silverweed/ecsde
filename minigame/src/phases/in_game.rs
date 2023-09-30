use super::Phase_Args;
use crate::sprites::{self as anim_sprites, Anim_Sprite};
use inle_app::phases::{Game_Phase, Phase_Id, Phase_Transition};
use inle_gfx::render_window::Render_Window_Handle;
use inle_gfx::sprites::Sprite;
use inle_math::rect::Rect;
use inle_math::vector::{lerp_v, Vec2f};
use inle_win::window;
use std::ops::DerefMut;
use std::time::Duration;

#[derive(Default)]
pub struct In_Game {
    sprites: Vec<Anim_Sprite>,
}

impl In_Game {
    pub const PHASE_ID: Phase_Id = Phase_Id::new("in_game");
}

impl Game_Phase for In_Game {
    type Args = Phase_Args;

    fn on_start(&mut self, args: &mut Self::Args) {
        use inle_gfx::res::tex_path;

        let mut gs = args.game_state_mut();
        let mut res = args.game_res_mut();
        let (win_w, win_h) = gs.app_config.target_win_size;

        // Currently we create the whole scene once and then we keep it in memory forever.
        if self.sprites.is_empty() {
            let tex_rect = inle_math::rect::Rect::new(0, 0, win_w as _, win_h as _);

            let gres = &mut res.gfx;
            let env = &gs.env;

            let tex_p = tex_path(env, "game/sky.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.z_index = -2;
            self.sprites.push(sprite.into());

            let tex_p = tex_path(env, "game/sky_mountains_background_.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.z_index = -1;
            self.sprites.push(sprite.into());

            let tex_p = tex_path(env, "game/terrain.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.rect = tex_rect;
            sprite.transform.translate(0., 200.);
            self.sprites.push(sprite.into());
        }
    }

    fn update(&mut self, args: &mut Self::Args) -> Phase_Transition {
        let mut game_state = args.game_state_mut();
        let gs = game_state.deref_mut();
        let game_res = args.game_res();

        let dt = gs.time.dt().as_secs_f32();
        for sprite in &self.sprites {
            anim_sprites::render_anim_sprite(&mut gs.window, &mut gs.batches, sprite);
        }
        Phase_Transition::None
    }
}
