use super::Phase_Args;
use crate::sprites::{self as anim_sprites, Anim_Sprite};
use inle_app::phases::{Game_Phase, Phase_Id, Phase_Transition};
use inle_gfx::render::Z_Index;
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

const Z_SKY: Z_Index = -2;
const Z_BG: Z_Index = -1;
const Z_MOUNTAINS: Z_Index = 0;
const Z_TERRAIN: Z_Index = 1;

impl Game_Phase for In_Game {
    type Args = Phase_Args;

    fn on_start(&mut self, args: &mut Self::Args) {
        use inle_gfx::res::tex_path;

        let mut gs = args.game_state_mut();
        let mut res = args.game_res_mut();
        let (win_w, win_h) = gs.app_config.target_win_size;
        let (win_hw, win_hh) = (win_w as f32 * 0.5, win_h as f32 * 0.5);

        // Currently we create the whole scene once and then we keep it in memory forever.
        if self.sprites.is_empty() {
            let win_rect = inle_math::rect::Rect::new(0, 0, win_w as _, win_h as _);

            let gres = &mut res.gfx;
            let env = &gs.env;

            let tex_p = tex_path(env, "game/sky.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.z_index = Z_SKY;
            self.sprites.push(sprite.into());

            let tex_p = tex_path(env, "game/sky_mountains_background_.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.z_index = Z_BG;
            sprite.color.a = 120;
            self.sprites.push(sprite.into());

            let tex_p = tex_path(env, "game/terrain.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.z_index = Z_TERRAIN;
            sprite.rect = win_rect;
            sprite.transform.translate(0., 270.);
            self.sprites.push(sprite.into());

            let mountain_off_x = win_hw - 90.;
            let mountain_off_y = 280.;
            let tex_p = tex_path(env, "game/mountain_bottom.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.transform.translate(-mountain_off_x, mountain_off_y);
            sprite.z_index = Z_MOUNTAINS;
            let y = sprite.transform.position().y;
            let h = sprite.rect.height as f32;
            let mut other = sprite.clone();
            other.transform.translate(2. * mountain_off_x, 0.);
            sprite.transform.set_scale(-1., 1.);
            self.sprites.push(sprite.into());
            self.sprites.push(other.into());

            let tex_p = tex_path(env, "game/mountain_center.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.transform.translate(-mountain_off_x, y - h + 1.);
            sprite.z_index = Z_MOUNTAINS;
            let mut other = sprite.clone();
            other.transform.translate(2. * mountain_off_x, 0.);
            sprite.transform.set_scale(-1., 1.);
            self.sprites.push(sprite.into());
            self.sprites.push(other.into());

            let tex_p = tex_path(env, "game/mountain_top_eyes_animation.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.transform.translate(-mountain_off_x, y - 2. * h + 2.);
            sprite.z_index = Z_MOUNTAINS;
            let mut sprite = Anim_Sprite::from_sprite(sprite, (2, 2), Duration::from_millis(170));
            let mut other = sprite.clone();
            other.transform.translate(2. * mountain_off_x, 0.);
            sprite.sprite.transform.set_scale(-1., 1.);
            self.sprites.push(sprite);
            self.sprites.push(other);
        }
    }

    fn update(&mut self, args: &mut Self::Args) -> Phase_Transition {
        let mut game_state = args.game_state_mut();
        let gs = game_state.deref_mut();
        let game_res = args.game_res();

        let dt = gs.time.dt().as_secs_f32();

        anim_sprites::update_anim_sprites(gs.time.dt(), &mut self.sprites);

        for sprite in &self.sprites {
            anim_sprites::render_anim_sprite(&mut gs.window, &mut gs.batches, sprite);
        }
        Phase_Transition::None
    }
}
