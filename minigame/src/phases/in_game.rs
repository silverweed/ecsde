use super::Phase_Args;
use crate::entity::{
    Entity, Entity_Container, Entity_Handle, Game_Collision_Layer as GCL, Phys_Type,
};
use crate::game::Game_State;
use crate::sprites::{self as anim_sprites, Anim_Sprite};
use inle_app::phases::{Game_Phase, Phase_Id, Phase_Transition};
use inle_cfg::{Cfg_Var, Config};
use inle_common::stringid::String_Id;
use inle_core::env::Env_Info;
use inle_gfx::render::Z_Index;
use inle_gfx::res::tex_path;
use inle_gfx::res::Gfx_Resources;
use inle_gfx::sprites::Sprite;
use inle_input::input_state::Action_Kind;
use inle_math::vector::Vec2f;
use inle_physics::collider::{Collider, Collision_Shape, Phys_Data};
use inle_physics::phys_world::{Collider_Handle, Physics_World};
use inle_physics::physics;
use std::ops::DerefMut;
use std::time::Duration;

mod blocks;

pub(super) struct Entity_Phys_Cfg {
    pub accel: Cfg_Var<f32>,
    //pub horiz_max_speed: Cfg_Var<f32>,
    //pub vert_max_speed: Cfg_Var<f32>,
    pub horiz_dampening: Cfg_Var<f32>,
    pub ascending_vert_dampening: Cfg_Var<f32>,
    pub descending_vert_dampening: Cfg_Var<f32>,
    pub gravity: Cfg_Var<f32>,
    // For blocks this is the throw impulse
    pub jump_impulse: Cfg_Var<f32>,
}

impl Entity_Phys_Cfg {
    pub fn new(cfg: &Config, cfg_path: &str) -> Self {
        let accel = Cfg_Var::<f32>::new(&format!("{}/acceleration", cfg_path), cfg);
        //    let horiz_max_speed =
        //        Cfg_Var::<f32>::new(&format!("{}/horiz_max_speed", cfg_path), cfg);
        //    let vert_max_speed =
        //        Cfg_Var::<f32>::new(&format!("{}/vert_max_speed", cfg_path), cfg);
        let horiz_dampening = Cfg_Var::<f32>::new(&format!("{}/horiz_dampening", cfg_path), cfg);
        let ascending_vert_dampening =
            Cfg_Var::<f32>::new(&format!("{}/ascending_vert_dampening", cfg_path), cfg);
        let descending_vert_dampening =
            Cfg_Var::<f32>::new(&format!("{}/descending_vert_dampening", cfg_path), cfg);
        let gravity = Cfg_Var::<f32>::new(&format!("{}/gravity", cfg_path), cfg);
        let jump_impulse = Cfg_Var::<f32>::new(&format!("{}/jump_impulse", cfg_path), cfg);
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

#[derive(Default)]
struct Player_Input {
    pub movement: Vec2f,
}

type Players_Input = [Player_Input; NUM_PLAYERS];

#[derive(Default)]
struct Player {
    pub entity: Entity_Handle,
    pub jumping: bool,
}

impl From<Entity_Handle> for Player {
    fn from(entity: Entity_Handle) -> Self {
        Self {
            entity,
            ..Default::default()
        }
    }
}

const NUM_PLAYERS: usize = 2;

pub struct In_Game {
    entities: Entity_Container,
    players: [Player; NUM_PLAYERS],
    houses: [Collider_Handle; NUM_PLAYERS],

    player_cfg: Entity_Phys_Cfg,
    phys_settings: physics::Physics_Settings,

    block_system: blocks::Block_System,
}

impl In_Game {
    pub const PHASE_ID: Phase_Id = Phase_Id::new("in_game");
}

pub(super) const Z_SKY: Z_Index = -3;
pub(super) const Z_BG: Z_Index = -2;
pub(super) const Z_HOUSES: Z_Index = -1;
pub(super) const Z_MOUNTAINS: Z_Index = 0;
pub(super) const Z_TERRAIN: Z_Index = 1;
pub(super) const Z_BLOCKS: Z_Index = 2;
pub(super) const Z_PLAYERS: Z_Index = 3;

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
        let cfg = &gs.config;

        let tex_p = tex_path(env, "game/sky.png");
        let mut sprite = Sprite::from_tex_path(gres, &tex_p);
        sprite.z_index = Z_SKY;
        self.entities.push(Entity::new(sprite.into()));

        let tex_p = tex_path(env, "game/sky_mountains_background_.png");
        let mut sprite = Sprite::from_tex_path(gres, &tex_p);
        sprite.z_index = Z_BG;
        sprite.color.a = 120;
        self.entities.push(Entity::new(sprite.into()));

        self.entities
            .push(create_boundaries(gs.app_config.target_win_size, physw, cfg));

        let terrain = create_terrain(env, gres, physw, cfg);
        self.entities.push(terrain);

        // Mountains
        let mountain_off_x = win_hw - 100.;
        let mountain_off_y = 280.;

        let mut left_mountain = create_mountain(env, gres, physw, cfg);
        let mut right_mountain = left_mountain.clone(physw);
        left_mountain
            .transform
            .translate(-mountain_off_x, mountain_off_y);
        left_mountain.transform.set_scale(-1., 1.);
        right_mountain
            .transform
            .translate(mountain_off_x, mountain_off_y);

        let mountain_height = left_mountain.sprites[0].rect.height * 3;

        self.entities.push(left_mountain);
        self.entities.push(right_mountain);

        let player_y = mountain_off_y - mountain_height as f32;
        let house_y = player_y + 80.;

        // Houses
        let mut house1 = create_house(env, gres, physw, "game/house_cyclops.png", cfg);
        house1.transform.set_position(-mountain_off_x, house_y);
        self.houses[0] = physw
            .get_physics_body(house1.phys_body)
            .unwrap()
            .all_colliders()
            .next()
            .unwrap();
        self.entities.push(house1);

        let mut house2 = create_house(env, gres, physw, "game/house_evil.png", cfg);
        house2.transform.set_position(mountain_off_x, house_y);
        self.houses[1] = physw
            .get_physics_body(house2.phys_body)
            .unwrap()
            .all_colliders()
            .next()
            .unwrap();
        self.entities.push(house2);

        // Players
        let mut player = create_player(env, gres, physw, cfg);
        player.sprites[0].color = inle_common::colors::color_from_hex_no_alpha(0xdd9884);
        player.transform.set_position(-mountain_off_x, player_y);
        self.players[0] = self.entities.push(player).into();

        let mut player = create_player(env, gres, physw, cfg);
        player.sprites[0].color = inle_common::colors::color_from_hex_no_alpha(0x1e98ff);
        player.transform.set_position(mountain_off_x, player_y);
        self.players[1] = self.entities.push(player).into();

        // Blocks
        self.block_system
            .init(100, &mut self.entities, gs, &mut res);
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

        if gs.time.paused {
            return Phase_Transition::None;
        }

        for action in &gs.input.processed.game_actions {
            match action {
                (name, Action_Kind::Pressed) if *name == sid!("open_pause_menu") => {
                    return Phase_Transition::Push(super::Pause_Menu::PHASE_ID);
                }
                _ => {}
            }
        }

        self.block_system.update(gs, &mut self.entities);

        let players_input = Self::read_players_input(gs);
        self.update_players(gs, &players_input);
        self.update_anim_states(gs, &players_input);

        anim_sprites::update_anim_sprites(
            gs.time.dt(),
            self.entities.iter_mut().flat_map(|e| &mut e.sprites),
        );

        self.update_phys_world(gs);
        self.check_win(gs);

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
    collision_matrix.set_layers_collide(GCL::Player, GCL::Houses);
    collision_matrix.set_layers_collide(GCL::Player, GCL::Boundary);
    collision_matrix.set_layers_collide(GCL::Player, GCL::Blocks);
    collision_matrix.set_layers_collide(GCL::Blocks, GCL::Blocks);
    collision_matrix.set_layers_collide(GCL::Blocks, GCL::Terrain);
    collision_matrix
}

impl In_Game {
    pub fn new(cfg: &Config) -> Self {
        let collision_matrix = create_collision_matrix();
        let positional_correction_percent =
            Cfg_Var::new("game/physics/positional_correction_percent", cfg);
        let phys_settings = physics::Physics_Settings {
            collision_matrix,
            positional_correction_percent,
        };
        let block_system = blocks::Block_System::new(cfg);

        Self {
            player_cfg: Entity_Phys_Cfg::new(cfg, "game/player"),
            entities: Entity_Container::default(),
            players: Default::default(),
            houses: Default::default(),
            phys_settings,
            block_system,
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
                    collider.velocity = entity.velocity;
                }
            }
        }

        inle_physics::physics::update_collisions(
            &self.entities,
            physw,
            &self.phys_settings,
            None,
            &mut game_state.frame_alloc,
            &game_state.config,
            #[cfg(debug_assertions)]
            &mut game_state.phys_debug_data,
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
                    entity.velocity = collider.velocity;
                    if entity.velocity.magnitude2() < 100. {
                        entity.velocity = v2!(0., 0.);
                    }
                }
            }
        }
    }

    fn read_players_input(game_state: &Game_State) -> Players_Input {
        let mut players_input = Players_Input::default();
        for player_idx in 0..NUM_PLAYERS {
            players_input[player_idx].movement = if game_state.free_camera {
                v2!(0., 0.)
            } else {
                let axes = [
                    String_Id::from(format!("p{}_horizontal", player_idx + 1).as_str()),
                    String_Id::from(format!("p{}_vertical", player_idx + 1).as_str()),
                ];
                let mut movement = crate::input::get_normalized_movement_from_input(
                    &game_state.input.processed.virtual_axes,
                    axes,
                    &game_state.input_cfg,
                    &game_state.config,
                );
                // No vertical movement
                movement.y = 0.;

                movement
            };
        }
        players_input
    }

    fn update_players(&mut self, game_state: &mut Game_State, players_input: &Players_Input) {
        let input = &game_state.input;
        let cfg = &game_state.config;
        let p_cfg = &self.player_cfg;
        let accel_magn = p_cfg.accel.read(cfg);
        let g = p_cfg.gravity.read(cfg);
        let jump_impulse = p_cfg.jump_impulse.read(cfg);
        let horiz_dampening = p_cfg.horiz_dampening.read(cfg);
        let dt = game_state.time.dt_secs();
        let physw = &game_state.phys_world;

        for (player_idx, player_state) in self.players.iter_mut().enumerate() {
            let player = self.entities.get_mut(player_state.entity).unwrap();
            let movement = players_input[player_idx].movement;

            // acceleration
            let mut accel = movement * accel_magn;

            // gravity
            let cld = physw
                .get_first_rigidbody_collider(player.phys_body)
                .unwrap();
            let mut collided = physw.get_collisions(cld.handle).iter().filter_map(|data| {
                Some((physw.get_collider(data.other_collider)?, data.info.normal))
            });
            let collides_with_ground = collided
                .any(|(other, normal)| normal.y < -0.9 && other.layer == GCL::Terrain as u8);
            if !collides_with_ground {
                accel += v2!(0., g);
            } else {
                player_state.jumping = false;
            }

            player.velocity += accel * dt;

            // jump
            if !game_state.free_camera
                && input.processed.game_actions.contains(&(
                    String_Id::from(format!("p{}_jump", player_idx + 1).as_str()),
                    Action_Kind::Pressed,
                ))
            {
                player.velocity += v2!(0., -jump_impulse);
                player_state.jumping = true;
            }

            // dampening
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

    fn update_anim_states(&mut self, game_state: &mut Game_State, players_input: &Players_Input) {
        for (player_idx, player_state) in self.players.iter().enumerate() {
            let player = self.entities.get_mut(player_state.entity).unwrap();
            let movement = players_input[player_idx].movement;

            match movement.x.partial_cmp(&0.) {
                Some(std::cmp::Ordering::Greater) => player.transform.set_scale(-1., 1.),
                Some(std::cmp::Ordering::Less) => player.transform.set_scale(1., 1.),
                _ => {}
            }

            let sprite = &mut player.sprites[0];
            if player_state.jumping {
                sprite.play(sid!("jump"));
            } else if movement.x.abs() > f32::EPSILON {
                sprite.play(sid!("walk"));
            } else {
                sprite.play(sid!("idle"));
            }
        }
    }

    fn check_win(&mut self, game_state: &mut Game_State) {
        let mut player_won = [false; NUM_PLAYERS];
        let physw = &game_state.phys_world;
        for (player_idx, player_state) in self.players.iter().enumerate() {
            let player = self.entities.get(player_state.entity).unwrap();
            let cld = physw
                .get_first_rigidbody_collider(player.phys_body)
                .unwrap();
            let collided_house = physw
                .get_collisions(cld.handle)
                .iter()
                .find(|data| {
                    if let Some(oth) = physw.get_collider(data.other_collider) {
                        if oth.layer == GCL::Houses.into() {
                            return true;
                        }
                    }
                    false
                })
                .map(|data| data.other_collider);

            if let Some(house) = collided_house {
                if house == self.houses[(player_idx + 1) % 2] {
                    player_won[player_idx] = true;
                }
            }
        }

        // TODO: handle win
    }
}

fn create_terrain(
    env: &Env_Info,
    gres: &mut Gfx_Resources,
    phys_world: &mut Physics_World,
    cfg: &Config,
) -> Entity {
    let tex_p = tex_path(env, "game/terrain.png");
    let mut sprite = Sprite::from_tex_path(gres, &tex_p);
    sprite.z_index = Z_TERRAIN;
    let mut terrain = Entity::new(sprite.into());
    terrain.transform.translate(0., 450.);

    let phys_data = Phys_Data {
        inv_mass: Cfg_Var::new_from_val(0.),
        restitution: Cfg_Var::new("game/physics/restitution", cfg),
        static_friction: Cfg_Var::new("game/physics/static_friction", cfg),
        dyn_friction: Cfg_Var::new("game/physics/dyn_friction", cfg),
    };
    terrain.register_to_physics(phys_world, &phys_data, GCL::Terrain, Phys_Type::Static);

    terrain
}

fn create_mountain(
    env: &Env_Info,
    gres: &mut Gfx_Resources,
    phys_world: &mut Physics_World,
    cfg: &Config,
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
    mountain.sprites.push(sprite);

    let phys_data = Phys_Data {
        inv_mass: Cfg_Var::new_from_val(0.),
        restitution: Cfg_Var::new("game/physics/restitution", cfg),
        static_friction: Cfg_Var::new("game/physics/static_friction", cfg),
        dyn_friction: Cfg_Var::new("game/physics/dyn_friction", cfg),
    };
    mountain.register_to_physics(phys_world, &phys_data, GCL::Terrain, Phys_Type::Static);

    mountain
}

fn create_player(
    env: &Env_Info,
    gres: &mut Gfx_Resources,
    phys_world: &mut Physics_World,
    cfg: &Config,
) -> Entity {
    let tex_p = tex_path(env, "game/player_white.png");
    let mut sprite = Sprite::from_tex_path(gres, &tex_p);
    sprite.z_index = Z_PLAYERS;
    let mut sprite = Anim_Sprite::from_sprite_with_size(sprite, v2!(64, 64));

    let d = Duration::from_millis(100);
    // TODO: some anims should not be looped
    sprite.add_animation(sid!("walk"), (0, 0), (4, 0), d);
    sprite.add_animation(sid!("idle"), (4, 0), (8, 0), d);
    sprite.add_animation(sid!("grab_walk"), (8, 0), (12, 0), d);
    sprite.add_animation(sid!("grab"), (0, 1), (4, 1), d);
    sprite.add_animation(sid!("throw"), (4, 1), (6, 1), d);
    sprite.add_animation(sid!("jump"), (6, 1), (11, 1), d);
    sprite.play(sid!("idle"));

    let mut player = Entity::new(sprite);
    let phys_data = Phys_Data {
        inv_mass: Cfg_Var::new("game/physics/player/mass", cfg),
        restitution: Cfg_Var::new("game/physics/restitution", cfg),
        static_friction: Cfg_Var::new("game/physics/static_friction", cfg),
        dyn_friction: Cfg_Var::new("game/physics/dyn_friction", cfg),
    };
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

fn create_boundaries(
    (win_w, win_h): (u32, u32),
    phys_world: &mut Physics_World,
    cfg: &Config,
) -> Entity {
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
        layer: GCL::Boundary.into(),
        is_static: true,
        offset: v2!((win_w as f32) * 0.5 + 250., 0.),
        ..Default::default()
    };

    let cld_top = Collider {
        shape: Collision_Shape::Rect {
            width: 2. * (win_w as f32),
            height: 500.,
        },
        layer: GCL::Terrain.into(),
        is_static: true,
        offset: v2!(0., -(win_h as f32) * 0.5 - 250.),
        ..Default::default()
    };

    let phys_data = Phys_Data {
        inv_mass: Cfg_Var::new_from_val(0.),
        restitution: Cfg_Var::new("game/physics/restitution", cfg),
        static_friction: Cfg_Var::new("game/physics/static_friction", cfg),
        dyn_friction: Cfg_Var::new("game/physics/dyn_friction", cfg),
    };
    let phys_body_hdl = phys_world
        .new_physics_body_with_rigidbodies([cld_left, cld_right, cld_top].into_iter(), phys_data);

    Entity {
        phys_body: phys_body_hdl,
        ..Default::default()
    }
}

fn create_house(
    env: &Env_Info,
    gres: &mut Gfx_Resources,
    phys_world: &mut Physics_World,
    path: &str,
    cfg: &Config,
) -> Entity {
    let tex_p = tex_path(env, path);
    let mut sprite = Sprite::from_tex_path(gres, &tex_p);
    sprite.z_index = Z_HOUSES;
    let cld = Collider {
        shape: Collision_Shape::Rect {
            width: sprite.rect.width as _,
            height: sprite.rect.height as _,
        },
        layer: GCL::Houses.into(),
        is_static: true,
        ..Default::default()
    };

    let mut house = Entity::new(sprite.into());

    let cld = phys_world.add_collider(cld);
    let phys_body_hdl = phys_world.new_physics_body();
    let phys_body = phys_world.get_physics_body_mut(phys_body_hdl).unwrap();
    phys_body.add_collider(cld);
    house.phys_body = phys_body_hdl;

    house
}
