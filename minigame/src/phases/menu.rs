use super::Phase_Args;
use inle_app::phases::{Game_Phase, Phase_Id, Phase_Transition};
use inle_core::rand;
use inle_gfx::render_window::Render_Window_Handle;
use inle_gfx::sprites::{self, Sprite};
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

struct Falling_Block {
    pub sprite: Sprite,
    pub speed: f32,
}

#[derive(Default)]
pub struct Main_Menu {
    buttons: Vec<Menu_Button>,

    sprites: Vec<Sprite>,
    block_sprites: Vec<Sprite>,
    falling_blocks: Vec<Falling_Block>,
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

fn new_falling_block(
    sprites: &[Sprite],
    win_size: (u32, u32),
    rng: &mut rand::Default_Rng,
) -> Falling_Block {
    const FALLING_BLOCK_SPEED_RANGE: (f32, f32) = (100., 250.);

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
    // 50% chance to be drawn atop of the mountains
    let z_index = if rand::rand_01(rng) < 0.5 { -1 } else { 1 };
    let mut block = sprites[block_type].clone();
    block.transform.translate(
        rand::rand_range(rng, -win_w, win_w),
        rand::rand_range(rng, -win_h * 2.0, -win_h),
    );
    block
        .transform
        .rotate(angle::rad(rand::rand_range(rng, 0., angle::TAU)));
    block.z_index = z_index;
    let speed = rand::rand_range(
        rng,
        FALLING_BLOCK_SPEED_RANGE.0,
        FALLING_BLOCK_SPEED_RANGE.1,
    );
    Falling_Block {
        sprite: block,
        speed,
    }
}

impl Game_Phase for Main_Menu {
    type Args = Phase_Args;

    fn on_start(&mut self, args: &mut Self::Args) {
        use inle_resources::gfx::tex_path;

        let mut gs = args.game_state_mut();
        let mut res = args.game_res_mut();
        let (win_w, win_h) = gs.app_config.target_win_size;

        if self.sprites.is_empty() {
            let tex_rect = inle_math::rect::Rect::new(0, 0, win_w as _, win_h as _);

            let gres = &mut res.gfx;
            let env = &gs.env;

            let tex_p = tex_path(env, "menu/main_menu_background.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.z_index = -1;
            self.sprites.push(sprite);

            let tex_p = tex_path(env, "menu/main_menu_mountains.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.rect = tex_rect;
            self.sprites.push(sprite);

            let tex_p = tex_path(env, "menu/game_logo.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            sprite.transform.translate(0., -200.);
            self.sprites.push(sprite);

            let tex_p = tex_path(env, "game/sun.png");
            let mut sprite = Sprite::from_tex_path(gres, &tex_p);
            self.sprites.push(sprite);

            // Block Sprites
            debug_assert!(self.block_sprites.is_empty());

            self.block_sprites.push(Sprite::from_tex_path(
                gres,
                &tex_path(env, "block/block_standard.png"),
            ));
            self.block_sprites.push(Sprite::from_tex_path(
                gres,
                &tex_path(env, "block/block_annoyed.png"),
            ));
            self.block_sprites.push(Sprite::from_tex_path(
                gres,
                &tex_path(env, "block/block_angry.png"),
            ));
            self.block_sprites.push(Sprite::from_tex_path(
                gres,
                &tex_path(env, "block/block_dummy.png"),
            ));

            // Spawn falling blocks
            const N_FALLING_BLOCKS: usize = 10;

            debug_assert!(self.falling_blocks.is_empty());
            self.falling_blocks.reserve_exact(N_FALLING_BLOCKS);

            for _i in 0..N_FALLING_BLOCKS {
                self.falling_blocks.push(new_falling_block(
                    &self.block_sprites,
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

        let dt = gs.time.dt();

        for button in &mut self.buttons {
            button.ease_t += dt.as_secs_f32();
        }

        let gres = &game_res.gfx;

        //
        // Draw background
        //
        for sprite in &self.sprites {
            inle_gfx::sprites::render_sprite(&mut gs.window, &mut gs.batches, sprite);
        }

        //
        // Update and draw falling blocks
        //
        let win_x = gs.app_config.target_win_size.0 as f32 * 0.5;
        let win_h = gs.app_config.target_win_size.1 as f32 * 0.5;
        for block in &mut self.falling_blocks {
            block
                .sprite
                .transform
                .translate(0., block.speed * dt.as_secs_f32());
            if block.sprite.transform.position().y > win_h + 20. {
                let mut new_block = new_falling_block(
                    &self.block_sprites,
                    gs.app_config.target_win_size,
                    &mut gs.rng,
                );
                std::mem::swap(&mut new_block, block);
            }
            inle_gfx::sprites::render_sprite(&mut gs.window, &mut gs.batches, &block.sprite);
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
