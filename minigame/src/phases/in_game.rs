use super::Phase_Args;
use crate::entity::{
    Entity, Entity_Container, Entity_Handle, Game_Collision_Layer as GCL, Phys_Type,
};
use crate::game::Game_State;
use crate::sprites::{self as anim_sprites, Anim_Sprite};
use inle_app::phases::{Game_Phase, Phase_Id, Phase_Transition};
use inle_cfg::Cfg_Var;
use inle_core::env::Env_Info;
use inle_gfx::render::Z_Index;
use inle_gfx::res::tex_path;
use inle_gfx::res::Gfx_Resources;
use inle_gfx::sprites::Sprite;
use inle_input::input_state::Action_Kind;
use inle_physics::collider::{Collider, Collision_Shape, Phys_Data};
use inle_physics::phys_world::Physics_World;
use inle_physics::physics;
use std::ops::DerefMut;
use std::time::Duration;

struct Player_Cfg {
    pub accel: Cfg_Var<f32>,
    //pub horiz_max_speed: Cfg_Var<f32>,
    //pub vert_max_speed: Cfg_Var<f32>,
    pub horiz_dampening: Cfg_Var<f32>,
    pub ascending_vert_dampening: Cfg_Var<f32>,
    pub descending_vert_dampening: Cfg_Var<f32>,
    pub gravity: Cfg_Var<f32>,
    pub jump_impulse: Cfg_Var<f32>,
}

impl Player_Cfg {
    pub fn new(cfg: &inle_cfg::Config) -> Self {
        let accel = inle_cfg::Cfg_Var::<f32>::new("game/gameplay/player/acceleration", cfg);
        //    let horiz_max_speed =
        //        inle_cfg::Cfg_Var::<f32>::new("game/gameplay/player/horiz_max_speed", cfg);
        //    let vert_max_speed =
        //        inle_cfg::Cfg_Var::<f32>::new("game/gameplay/player/vert_max_speed", cfg);
        let horiz_dampening =
            inle_cfg::Cfg_Var::<f32>::new("game/gameplay/player/horiz_dampening", cfg);
        let ascending_vert_dampening =
            inle_cfg::Cfg_Var::<f32>::new("game/gameplay/player/ascending_vert_dampening", cfg);
        let descending_vert_dampening =
            inle_cfg::Cfg_Var::<f32>::new("game/gameplay/player/descending_vert_dampening", cfg);
        let gravity = inle_cfg::Cfg_Var::<f32>::new("game/gameplay/player/gravity", cfg);
        let jump_impulse = inle_cfg::Cfg_Var::<f32>::new("game/gameplay/player/jump_impulse", cfg);
        Self {
            accel,
            //horiz_max_speed,
            //vert_max_speed,
            horiz_dampening,
            ascending_vert_dampening,
            descending_vert_dampening,
            gravity,
            jump_impulse,
        }
    }
}

pub struct In_Game {
    entities: Entity_Container,
    players: [Entity_Handle; 2],
    player_cfg: Player_Cfg,
    phys_settings: physics::Physics_Settings,
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
        let gs = game_state.deref_mut();
        let mut res = args.game_res_mut();
        let win_w = gs.app_config.target_win_size.0;
        let win_hw = win_w as f32 * 0.5;

        debug_assert_eq!(self.entities.n_live(), 0);

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

        self.entities.push(create_boundaries(gs.app_config.target_win_size, physw));

        let terrain = create_terrain(env, gres, physw);
        self.entities.push(terrain);

        // Mountains
        let mountain_off_x = win_hw - 100.;
        let mountain_off_y = 280.;

        let mut left_mountain = create_mountain(env, gres, physw);
        let mut right_mountain = left_mountain.clone(physw);
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
        self.players[0] = self.entities.push(player);
    }

    fn on_end(&mut self, args: &mut Self::Args) {
        let mut game_state = args.game_state_mut();
        let gs = game_state.deref_mut();
        let physw = &mut gs.phys_world;
        physw.clear_all();
        self.entities.clear();
    }

    fn update(&mut self, args: &mut Self::Args) -> Phase_Transition {
        let mut game_state = args.game_state_mut();
        let gs = game_state.deref_mut();

        self.update_players(gs);

        /*
        gs.camera.transform.set_position_v(
            self.entities
                .get(self.players[0])
                .unwrap()
                .transform
                .position(),
        );
        */

        anim_sprites::update_anim_sprites(
            gs.time.dt(),
            self.entities.iter_mut().map(|e| &mut e.sprites).flatten(),
        );

        self.update_phys_world(gs);

        Phase_Transition::None
    }

    fn draw(&self, args: &mut Self::Args) {
        let mut game_state = args.game_state_mut();
        let gs = game_state.deref_mut();

        for entity in &self.entities {
            entity.draw(&mut gs.window, &mut gs.batches);
        }
    }
}

fn create_collision_matrix() -> inle_physics::layers::Collision_Matrix {
    let mut collision_matrix = inle_physics::layers::Collision_Matrix::default();
    collision_matrix.set_layers_collide(GCL::Player, GCL::Terrain);
    collision_matrix.set_layers_collide(GCL::Player, GCL::Player);
    collision_matrix
}

impl In_Game {
    pub fn new(cfg: &inle_cfg::Config) -> Self {
        let collision_matrix = create_collision_matrix();
        let positional_correction_percent =
            Cfg_Var::new("game/physics/positional_correction_percent", cfg);
        let phys_settings = physics::Physics_Settings {
            collision_matrix,
            positional_correction_percent,
        };

        Self {
            player_cfg: Player_Cfg::new(cfg),
            entities: Entity_Container::default(),
            players: Default::default(),
            phys_settings,
        }
    }

    fn update_phys_world(&mut self, game_state: &mut Game_State) {
        trace!("update_phys_world");

        let physw = &mut game_state.phys_world;

        // Copy entities' positions to colliders
        for entity in &self.entities {
            // @Speed: can we avoid cloning this?
            if let Some(phys_body) = physw.get_physics_body(entity.phys_body).cloned() {
                for cld_handle in phys_body.all_colliders() {
                    let collider = physw.get_collider_mut(cld_handle).unwrap();
                    // NOTE: currently we don't support fancy stuff like rotations for offset
                    // colliders.
                    collider.position = entity.transform.position() + collider.offset;
                }
            }
        }

        #[cfg(debug_assertions)]
        let mut debug_data = inle_physics::physics::Collision_System_Debug_Data::default();
        inle_physics::physics::update_collisions(
            &self.entities,
            physw,
            &self.phys_settings,
            None,
            &mut game_state.frame_alloc,
            &game_state.config,
            #[cfg(debug_assertions)]
            &mut debug_data,
        );

        // Copy back new colliders positions to entities
        for entity in &mut self.entities {
            // @Speed: can we avoid cloning this?
            if let Some(phys_body) = physw.get_physics_body(entity.phys_body).cloned() {
                for cld_handle in phys_body.all_colliders() {
                    let collider = physw.get_collider_mut(cld_handle).unwrap();
                    entity
                        .transform
                        .set_position_v(collider.position - collider.offset);
                }
            }
        }
    }

    fn update_players(&mut self, game_state: &mut Game_State) {
        let input = &game_state.input;
        let cfg = &game_state.config;

        let movement = if game_state.free_camera {
            v2!(0., 0.)
        } else {
            let mut movement = crate::input::get_normalized_movement_from_input(
                &input.processed.virtual_axes,
                &game_state.input_cfg,
                cfg,
            );
            // No vertical movement
            movement.y = 0.;

            movement
        };

        let p_cfg = &self.player_cfg;

        let dt = game_state.time.dt_secs();
        let player = self.entities.get_mut(self.players[0]).unwrap();

        // acceleration
        let accel_magn = p_cfg.accel.read(cfg);
        let mut accel = movement * accel_magn;

        // gravity
        let physw = &game_state.phys_world;
        let cld = physw
            .get_first_rigidbody_collider(player.phys_body)
            .unwrap();
        let mut collided = physw
            .get_collisions(cld.handle)
            .iter()
            .filter_map(|data| Some((physw.get_collider(data.other_collider)?, data.info.normal)));
        let collides_with_ground = collided
            .find(|(other, normal)| normal.y < -0.9 && other.layer == GCL::Terrain as u8)
            .is_some();
        if !collides_with_ground {
            let g = p_cfg.gravity.read(cfg);
            accel += v2!(0., g);
        }

        player.velocity += accel * dt;

        // jump
        if !game_state.free_camera
            && input
                .processed
                .game_actions
                .contains(&(sid!("jump"), Action_Kind::Pressed))
        {
            let jump_impulse = p_cfg.jump_impulse.read(cfg);
            player.velocity += v2!(0., -jump_impulse);
        }

        // dampening
        let horiz_dampening = p_cfg.horiz_dampening.read(cfg);
        let vert_dampening = if player.velocity.y > 0. {
            p_cfg.descending_vert_dampening.read(cfg)
        } else {
            p_cfg.ascending_vert_dampening.read(cfg)
        };
        player.velocity.x -= player.velocity.x * horiz_dampening * dt;
        player.velocity.y -= player.velocity.y * vert_dampening * dt;

        player.transform.translate_v(player.velocity * dt);
    }
}

const PHYS_RESTITUTION: f32 = 1.0;

fn create_terrain(
    env: &Env_Info,
    gres: &mut Gfx_Resources,
    phys_world: &mut Physics_World,
) -> Entity {
    let tex_p = tex_path(env, "game/terrain.png");
    let mut sprite = Sprite::from_tex_path(gres, &tex_p);
    sprite.z_index = Z_TERRAIN;
    let mut terrain = Entity::new(sprite.into());
    terrain.transform.translate(0., 450.);

    let phys_data = Phys_Data::default()
        .with_infinite_mass()
        .with_restitution(PHYS_RESTITUTION)
        .with_static_friction(0.5)
        .with_dyn_friction(0.3);
    terrain.register_to_physics(phys_world, &phys_data, GCL::Terrain, Phys_Type::Static);

    terrain
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

    let phys_data = Phys_Data::default()
        .with_infinite_mass()
        .with_restitution(PHYS_RESTITUTION)
        .with_static_friction(0.5)
        .with_dyn_friction(0.3);
    mountain.register_to_physics(phys_world, &phys_data, GCL::Terrain, Phys_Type::Static);

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
    let phys_data = Phys_Data::default()
        .with_mass(1.)
        .with_restitution(PHYS_RESTITUTION)
        .with_static_friction(0.5)
        .with_dyn_friction(0.3);

    let cld = Collider {
        shape: Collision_Shape::Rect {
            width: 28.,
            height: 54.,
        },
        layer: GCL::Player.into(),
        ..Default::default()
    };
    player.phys_body = phys_world.new_physics_body_with_rigidbody(cld, phys_data);

    player
}

fn create_boundaries((win_w, win_h): (u32, u32), phys_world: &mut Physics_World) -> Entity {
    let cld_left = Collider {
        shape: Collision_Shape::Rect {
            width: 500.,
            height: 2. * (win_h as f32),
        },
        layer: GCL::Terrain.into(),
        is_static: true,
        offset: v2!(-(win_w as f32) * 0.5 - 250., 0.),
        ..Default::default()
    };

    let cld_right = Collider {
        shape: Collision_Shape::Rect {
            width: 500.,
            height: 2. * (win_h as f32),
        },
        layer: GCL::Terrain.into(),
        is_static: true,
        offset: v2!((win_w as f32) * 0.5 + 250., 0.),
        ..Default::default()
    };

    let cld_top = Collider {
        shape: Collision_Shape::Rect {
            width: 2. * (win_w as f32),
            height: 500.
        },
        layer: GCL::Terrain.into(),
        is_static: true,
        offset: v2!(0., -(win_h as f32) * 0.5 - 250.),
        ..Default::default()
    };

    let phys_data = Phys_Data::default()
        .with_infinite_mass()
        .with_restitution(PHYS_RESTITUTION)
        .with_static_friction(0.5)
        .with_dyn_friction(0.3);
    let mut phys_body_hdl = phys_world.new_physics_body_with_rigidbodies([cld_left, cld_right, cld_top].into_iter(), phys_data);

    let mut entity = Entity::default();
    entity.phys_body = phys_body_hdl;

    entity
}
