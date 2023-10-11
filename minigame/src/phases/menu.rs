use super::Phase_Args;
use crate::sprites::{self as anim_sprites, Anim_Sprite};
use inle_app::phases::{Game_Phase, Phase_Id, Phase_Transition};
use inle_core::rand;
use inle_gfx::render_window::Render_Window_Handle;
use inle_gfx::sprites::Sprite;
use inle_math::angle;
use inle_math::rect::Rect;
use inle_math::vector::{lerp_v, Vec2f};
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
struct Falling_Block {
    pub sprite_idx: usize,
    pub speed: f32,
    pub ang_speed: angle::Angle,
}

#[derive(Default)]
pub struct Main_Menu {
    buttons: Vec<Menu_Button>,

    sprites: Vec<Anim_Sprite>,
    block_sprites_first_idx: usize, // idx into `sprites`
    falling_blocks: Vec<Falling_Block>,

    rays_sprite_idx: usize,
}

impl Main_Menu {
    pub const PHASE_ID: Phase_Id = Phase_Id::new("menu");

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
        let tgy = (wh - size.y) * 0.5 + 250.0;
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

fn replace_falling_block(
    // NOTE: sprites contains a bunch of long-lived sprites (background, sun, template falling
    // blocks) and a bunch of short-lived ones (the falling blocks). The total amount of sprites
    // never changes because falling blocks are created and destroyed simultaneously.
    sprites: &mut [Anim_Sprite],
    replaced_idx: usize, // this is the index (into `sprites`) we're replacing
    block_sprites_first_idx: usize,
    win_size: (u32, u32),
    rng: &mut rand::Default_Rng,
) -> Falling_Block {
    const FALLING_BLOCK_SPEED_RANGE: (f32, f32) = (100., 250.);
    const FALLING_BLOCK_ANG_SPEED_DEG: f32 = 200.;

    let win_w = win_size.0 as f32 * 0.5;
    let win_h = win_size.1 as f32 * 0.5;
    let bt_rand = rand::rand_01(rng);
    let block_type = if bt_rand < 0.1 {
        1
    } else if bt_rand < 0.2 {
        2
    } else if bt_rand < 0.3 {
        3
    } else {
        0
    };

    sprites[replaced_idx] = sprites[block_sprites_first_idx + block_type].clone();
    let block = &mut sprites[replaced_idx];

    // Initial random position + rotation
    block.transform.translate(
        rand::rand_range(rng, -win_w..win_w),
        rand::rand_range(rng, -win_h * 2.0..-win_h),
    );
    block
        .transform
        .rotate(angle::rad(rand::rand_range(rng, 0.0..angle::TAU)));
    //
    // 50% chance to be drawn atop of the mountains
    let z_index = if rand::rand_01(rng) < 0.5 { -1 } else { 1 };
    block.z_index = z_index;

    let speed = rand::rand_range(
        rng,
        FALLING_BLOCK_SPEED_RANGE.0..FALLING_BLOCK_SPEED_RANGE.1,
    );
    let ang_speed = angle::deg(rand::rand_range(
        rng,
        -FALLING_BLOCK_ANG_SPEED_DEG..FALLING_BLOCK_ANG_SPEED_DEG,
    ));
    Falling_Block {
        sprite_idx: replaced_idx,
        speed,
        ang_speed,
    }
}

impl Game_Phase for Main_Menu {
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

            let tex_p = tex_path(env, "menu/main_menu_background.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.transform.set_scale(1.5, 1.5);
            sprite.z_index = -1;
            self.sprites.push(sprite.into());

            let tex_p = tex_path(env, "menu/main_menu_mountains.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.transform.set_scale(1.5, 1.5);
            sprite.rect = tex_rect;
            self.sprites.push(sprite.into());

            let tex_p = tex_path(env, "menu/game_logo.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.transform.set_scale(1.5, 1.5);
            sprite.transform.translate(0., -300.);
            sprite.z_index = 2;
            self.sprites.push(sprite.into());

            let tex_p = tex_path(env, "game/rays.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.transform.set_scale(2.5, 2.5);
            self.sprites.push(sprite.into());
            self.rays_sprite_idx = self.sprites.len() - 1;

            let tex_p = tex_path(env, "game/sun_eyes_animation.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.transform.set_scale(1.5, 1.5);
            sprite.z_index = 2;
            let anim_sprite = Anim_Sprite::from_sprite(sprite, (4, 2), Duration::from_millis(120));
            self.sprites.push(anim_sprite);

            // Block Sprites
            self.block_sprites_first_idx = self.sprites.len();
            self.sprites.push(Anim_Sprite::from_tex_path(
                gres,
                &tex_path(env, "block/block_standard.png"),
                (1, 1),
                Duration::default(),
            ));
            self.sprites.push(Anim_Sprite::from_tex_path(
                gres,
                &tex_path(env, "block/block_annoyed_eyes_animation.png"),
                (8, 1),
                Duration::from_millis(100),
            ));
            self.sprites.push(Anim_Sprite::from_tex_path(
                gres,
                &tex_path(env, "block/block_angry_eyes_animation.png"),
                (8, 1),
                Duration::from_millis(100),
            ));
            self.sprites.push(Anim_Sprite::from_tex_path(
                gres,
                &tex_path(env, "block/block_dummy_eyes_animation.png"),
                (8, 1),
                Duration::from_millis(100),
            ));

            // Spawn falling blocks
            const N_FALLING_BLOCKS: usize = 10;

            let first_falling_block_idx = self.sprites.len();

            debug_assert!(self.falling_blocks.is_empty());
            let block_template = self.sprites[first_falling_block_idx - 1].clone();
            self.sprites
                .resize(first_falling_block_idx + N_FALLING_BLOCKS, block_template);
            self.falling_blocks.reserve_exact(N_FALLING_BLOCKS);

            for i in 0..N_FALLING_BLOCKS {
                self.falling_blocks.push(replace_falling_block(
                    &mut self.sprites,
                    first_falling_block_idx + i,
                    self.block_sprites_first_idx,
                    gs.app_config.target_win_size,
                    &mut gs.rng,
                ));
            }
        }
    }

    fn update(&mut self, args: &mut Self::Args) -> Phase_Transition {
        let mut game_state = args.game_state_mut();
        let gs = game_state.deref_mut();
        let game_res = args.game_res();

        let dt = gs.time.dt().as_secs_f32();

        for button in &mut self.buttons {
            button.ease_t += dt;
        }

        anim_sprites::update_anim_sprites(gs.time.dt(), &mut self.sprites);

        let gres = &game_res.gfx;

        //
        // Rotate sun rays
        //
        {
            const RAYS_ANG_SPEED: angle::Angle = angle::rad(0.16);
            let rays = &mut self.sprites[self.rays_sprite_idx];
            rays.transform.rotate(dt * RAYS_ANG_SPEED);
        }

        //
        // Update falling blocks
        //
        let win_h = gs.app_config.target_win_size.1 as f32 * 0.5;
        for block in &mut self.falling_blocks {
            let sprite = &mut self.sprites[block.sprite_idx];
            sprite.transform.translate(0., block.speed * dt);
            sprite.transform.rotate(block.ang_speed * dt);
            if sprite.transform.position().y > win_h + 20. {
                let mut new_block = replace_falling_block(
                    &mut self.sprites,
                    block.sprite_idx,
                    self.block_sprites_first_idx,
                    gs.app_config.target_win_size,
                    &mut gs.rng,
                );
                std::mem::swap(&mut new_block, block);
            }
        }

        //
        // Draw sprites
        //
        for sprite in &self.sprites {
            anim_sprites::render_anim_sprite(&mut gs.window, &mut gs.batches, sprite);
        }

        //
        // Draw buttons
        //
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
            return Phase_Transition::Push(super::In_Game::PHASE_ID);
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
