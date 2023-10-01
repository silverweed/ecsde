use super::Phase_Args;
use crate::entity::{Entity, Entity_Container};
use crate::game::Game_State;
use crate::sprites::{self as anim_sprites, Anim_Sprite};
use inle_app::phases::{Game_Phase, Phase_Id, Phase_Transition};
use inle_core::env::Env_Info;
use inle_gfx::render::Z_Index;
use inle_gfx::render_window::Render_Window_Handle;
use inle_gfx::res::tex_path;
use inle_gfx::res::Gfx_Resources;
use inle_gfx::sprites::Sprite;
use inle_math::rect::Rect;
use inle_math::vector::{lerp_v, Vec2f};
use inle_physics::phys_world::Physics_World;
use inle_win::window;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

#[derive(Default)]
pub struct In_Game {
    entities: Entity_Container,
}

impl In_Game {
    pub const PHASE_ID: Phase_Id = Phase_Id::new("in_game");
}

const Z_SKY: Z_Index = -2;
const Z_BG: Z_Index = -1;
const Z_MOUNTAINS: Z_Index = 0;
const Z_TERRAIN: Z_Index = 1;
const Z_PLAYERS: Z_Index = 2;

impl Game_Phase for In_Game {
    type Args = Phase_Args;

    fn on_start(&mut self, args: &mut Self::Args) {
        let game_state = &mut args.game_state_mut();
        let mut gs = game_state.deref_mut();
        let mut res = args.game_res_mut();
        let (win_w, win_h) = gs.app_config.target_win_size;
        let (win_hw, win_hh) = (win_w as f32 * 0.5, win_h as f32 * 0.5);

        // Currently we create the whole scene once and then we keep it in memory forever.
        if self.entities.no_entity_was_ever_added() {
            let win_rect = inle_math::rect::Rect::new(0, 0, win_w as _, win_h as _);

            let gres = &mut res.gfx;
            let env = &gs.env;
            let physw = &mut gs.phys_world;

            let tex_p = tex_path(env, "game/sky.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.z_index = Z_SKY;
            self.entities.push(Entity::new(sprite.into()));

            let tex_p = tex_path(env, "game/sky_mountains_background_.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.z_index = Z_BG;
            sprite.color.a = 120;
            self.entities.push(Entity::new(sprite.into()));

            let tex_p = tex_path(env, "game/terrain.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.z_index = Z_TERRAIN;
            sprite.rect = win_rect;
            sprite.transform.translate(0., 270.);
            self.entities.push(Entity::new(sprite.into()));

            // Mountains
            let mountain_off_x = win_hw - 90.;
            let mountain_off_y = 280.;

            let mut left_mountain = create_mountain(env, gres, physw);
            let mut right_mountain = left_mountain.clone();
            left_mountain
                .transform
                .translate(-mountain_off_x, mountain_off_y);
            left_mountain.transform.set_scale(-1., 1.);
            right_mountain
                .transform
                .translate(mountain_off_x, mountain_off_y);

            self.entities.push(left_mountain);
            self.entities.push(right_mountain);

            // Players
            let mut player = create_player(env, gres, physw);
            player.sprites[0].color = inle_common::colors::color_from_hex_no_alpha(0xdd98844);
            self.entities.push(player);
        }
    }

    fn update(&mut self, args: &mut Self::Args) -> Phase_Transition {
        let mut game_state = args.game_state_mut();
        let gs = game_state.deref_mut();
        let game_res = args.game_res();

        let dt = gs.time.dt().as_secs_f32();

        anim_sprites::update_anim_sprites(
            gs.time.dt(),
            self.entities.iter_mut().map(|e| &mut e.sprites).flatten(),
        );

        self.update_phys_world(gs);

        for entity in &self.entities {
            entity.draw(&mut gs.window, &mut gs.batches);
        }

        Phase_Transition::None
    }
}

impl In_Game {
    fn update_phys_world(&mut self, game_state: &mut Game_State) {
        let physw = &mut game_state.phys_world;

        // sync colliders and entities positions
        for entity in &self.entities {
            // @Speed: can we avoid cloning this?
            if let Some(phys_body) = physw.get_physics_body(entity.phys_body).cloned() {
                for cld_handle in phys_body.all_colliders() {
                    let collider = physw.get_collider_mut(cld_handle).unwrap();
                    collider.position = entity.transform.position();
                }
            }
        }
    }
}

fn create_mountain(
    env: &Env_Info,
    gres: &mut Gfx_Resources,
    phys_world: &mut Physics_World,
) -> Entity {
    let tex_p = tex_path(env, "game/mountain_bottom.png");
    let mut sprite = Sprite::from_tex_path(gres, &tex_p);
    sprite.z_index = Z_MOUNTAINS;
    let y = sprite.transform.position().y;
    let h = sprite.rect.height as f32;

    let mut mountain = Entity::default();
    mountain.sprites.push(sprite.into());

    let tex_p = tex_path(env, "game/mountain_center.png");
    let mut sprite = Sprite::from_tex_path(gres, &tex_p);
    sprite.transform.translate(0., y - h + 1.);
    sprite.z_index = Z_MOUNTAINS;
    mountain.sprites.push(sprite.into());

    let tex_p = tex_path(env, "game/mountain_top_eyes_animation.png");
    let mut sprite = Sprite::from_tex_path(gres, &tex_p);
    sprite.transform.translate(0., y - 2. * h + 6.);
    sprite.z_index = Z_MOUNTAINS;
    let sprite = Anim_Sprite::from_sprite(sprite, (2, 2), Duration::from_millis(170));
    mountain.sprites.push(sprite.into());

    mountain.register_to_physics(phys_world);

    mountain
}

fn create_player(
    env: &Env_Info,
    gres: &mut Gfx_Resources,
    phys_world: &mut Physics_World,
) -> Entity {
    let tex_p = tex_path(env, "game/player_white.png");
    let mut sprite = Sprite::from_tex_path(gres, &tex_p);
    sprite.z_index = Z_PLAYERS;
    let mut sprite = Anim_Sprite::from_sprite_with_size(sprite, v2!(64, 64));

    let d = Duration::from_millis(100);
    sprite.add_animation(sid!("walk"), (0, 0), (4, 0), d);
    sprite.add_animation(sid!("idle"), (4, 0), (8, 0), d);
    sprite.add_animation(sid!("grab_walk"), (8, 0), (12, 0), d);
    sprite.add_animation(sid!("grab"), (0, 1), (4, 1), d);
    sprite.add_animation(sid!("throw"), (4, 1), (6, 1), d);
    sprite.add_animation(sid!("jump"), (6, 1), (11, 1), d);
    sprite.play(sid!("idle"));

    let mut player = Entity::new(sprite);
    player.register_to_physics(phys_world);

    player
}
