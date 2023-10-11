use crate::entity::{
    Entity, Entity_Container, Entity_Handle, Game_Collision_Layer as GCL, Phys_Type,
};
use crate::sprites::Anim_Sprite;
use crate::{Game_Resources, Game_State};
use inle_cfg::{Cfg_Var, Config};
use inle_core::env::Env_Info;
use inle_core::rand::Default_Rng;
use inle_gfx::res::{tex_path, Gfx_Resources};
use inle_math::vector::Vec2i;
use inle_physics::collider::Phys_Data;
use inle_physics::phys_world::Physics_World;
use std::time::Duration;

#[derive(Default)]
pub struct Block {
    pub entity: Entity_Handle,
}

pub struct Block_System {
    travelling_blocks: Vec<usize>,
    blocks: Vec<Block>,

    phys_cfg: super::Entity_Phys_Cfg,
}

impl Block_System {
    pub fn new(cfg: &Config) -> Self {
        Self {
            phys_cfg: super::Entity_Phys_Cfg::new(cfg, "game/block"),
            blocks: vec![],
            travelling_blocks: vec![],
        }
    }

    pub fn init(
        &mut self,
        n_blocks: usize,
        spawn_area_size: Vec2i,
        entities: &mut Entity_Container,
        game_state: &mut Game_State,
        game_res: &mut Game_Resources,
    ) {
        let rng = &mut game_state.rng;
        let env = &game_state.env;
        let gres = &mut game_res.gfx;
        let physw = &mut game_state.phys_world;
        let cfg = &game_state.config;

        self.blocks.clear();
        self.travelling_blocks.clear();

        self.blocks.reserve(n_blocks);
        self.travelling_blocks.reserve(n_blocks);

        let win_w = game_state.app_config.target_win_size.0 as f32;
        let win_h = game_state.app_config.target_win_size.1 as f32;
        let spawn_offset_x =
            (game_state.app_config.target_win_size.0 - spawn_area_size.x as u32) / 2;

        for i in 0..n_blocks {
            let mut entity = create_block(env, gres, physw, cfg, rng);
            let block_width = entity.sprites[0].sprite.rect.width;
            let x = (block_width as f32)
                * ((rng.next() % (spawn_area_size.x as u64 / block_width as u64)) as f32)
                + spawn_offset_x as f32
                - win_w * 0.5;
            let y = -win_h - ((n_blocks - i) * spawn_area_size.y as usize) as f32;
            entity.transform.set_position(x, y);
            let handle = entities.push(entity);
            let mut block = Block { entity: handle };
            self.blocks.push(block);
            self.travelling_blocks.push(i);
        }
    }

    pub fn update(&mut self, game_state: &mut Game_State, entities: &mut Entity_Container) {
        let gravity = self.phys_cfg.gravity.read(&game_state.config);
        let dt = game_state.time.dt_secs();
        let max_vspeed = self.phys_cfg.vert_max_speed.read(&game_state.config);
        let max_vspeed2 = max_vspeed * max_vspeed;
        let physw = &game_state.phys_world;

        for &block_idx in &self.travelling_blocks {
            let block = &self.blocks[block_idx];
            let block_ent = entities.get_mut(block.entity).unwrap();

            let collides_below = block_collides_from_below(physw, block_ent.phys_body);
            let mut accel = v2!(0., 0.);
            if collides_below {
                block_ent.velocity = v2!(0., 0.);
            } else {
                // Apply gravity
                accel = v2!(0., gravity);
                block_ent.velocity += accel * dt;
            }

            let vspeed2 = block_ent.velocity.magnitude2();
            if vspeed2 > max_vspeed2 {
                block_ent.velocity = max_vspeed * block_ent.velocity.normalized();
            }

            block_ent.transform.translate_v(block_ent.velocity * dt);
        }
    }
}

fn create_block(
    env: &Env_Info,
    gres: &mut Gfx_Resources,
    phys_world: &mut Physics_World,
    cfg: &Config,
    rng: &mut Default_Rng,
) -> Entity {
    let typ = ((rng.next() % 20) as u8).min(3).into();
    let mut sprite = make_block_sprite(env, gres, typ);
    sprite.z_index = super::Z_BLOCKS;

    let cld = inle_physics::collider::Collider {
        shape: inle_physics::collider::Collision_Shape::Rect {
            width: sprite.rect.width as _,
            height: sprite.rect.height as _,
        },
        layer: GCL::Blocks.into(),
        ..Default::default()
    };

    let mut entity = Entity::new(sprite);
    entity.transform.set_scale(1.886, 1.886);
    let phys_data = Phys_Data {
        inv_mass: Cfg_Var::new("game/physics/block/mass", cfg),
        restitution: Cfg_Var::new("game/physics/block/restitution", cfg),
        static_friction: Cfg_Var::new("game/physics/static_friction", cfg),
        dyn_friction: Cfg_Var::new("game/physics/dyn_friction", cfg),
    };
    entity.register_to_physics(phys_world, &phys_data, GCL::Blocks, Phys_Type::Dynamic);

    /*
    let cld = phys_world.add_collider(cld);
    let phys_body_hdl = phys_world.new_physics_body();
    let phys_body = phys_world.get_physics_body_mut(phys_body_hdl).unwrap();
    phys_body.add_collider(cld);
    entity.phys_body = phys_body_hdl;
    */

    entity
}

#[repr(u8)]
enum Block_Type {
    Standard,
    Annoyed,
    Angry,
    Dummy,
}

impl From<u8> for Block_Type {
    fn from(n: u8) -> Self {
        assert!(n < 4);
        unsafe { std::mem::transmute(n) }
    }
}

fn make_block_sprite(
    env: &Env_Info,
    gres: &mut Gfx_Resources,
    block_type: Block_Type,
) -> Anim_Sprite {
    match block_type {
        Block_Type::Standard => Anim_Sprite::from_tex_path(
            gres,
            &tex_path(env, "block/block_standard.png"),
            (1, 1),
            Duration::default(),
        ),
        Block_Type::Annoyed => Anim_Sprite::from_tex_path(
            gres,
            &tex_path(env, "block/block_annoyed_eyes_animation.png"),
            (8, 1),
            Duration::from_millis(100),
        ),
        Block_Type::Angry => Anim_Sprite::from_tex_path(
            gres,
            &tex_path(env, "block/block_angry_eyes_animation.png"),
            (8, 1),
            Duration::from_millis(100),
        ),
        Block_Type::Dummy => Anim_Sprite::from_tex_path(
            gres,
            &tex_path(env, "block/block_dummy_eyes_animation.png"),
            (8, 1),
            Duration::from_millis(100),
        ),
    }
}

fn block_collides_from_below(
    physw: &Physics_World,
    phys_body: inle_physics::phys_world::Physics_Body_Handle,
) -> bool {
    let cld = physw.get_all_phys_body_colliders(phys_body).next().unwrap();
    let mut collided = physw
        .get_collisions(cld.handle)
        .iter()
        .filter_map(|data| Some((physw.get_collider(data.other_collider)?, data.info.normal)));
    let collides = collided.any(|(other, normal)| normal.y < -0.9);

    collides
}
