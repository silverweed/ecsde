use crate::entity::{
    Entity, Entity_Container, Entity_Handle, Game_Collision_Layer as GCL, Phys_Type,
};
use crate::sprites::Anim_Sprite;
use crate::{Game_Resources, Game_State};
use inle_cfg::{Cfg_Var, Config};
use inle_core::env::Env_Info;
use inle_core::rand::Default_Rng;
use inle_gfx::res::{tex_path, Gfx_Resources};
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

        for i in 0..n_blocks {
            let mut entity = create_block(env, gres, physw, cfg, rng);
            let x = inle_core::rand::rand_range(rng, -0.5 * win_w..0.5 * win_w);
            let y = -win_h - (n_blocks - i) as f32 * 200.;
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

        for &block_idx in &self.travelling_blocks {
            let block = &self.blocks[block_idx];
            let block_ent = entities.get_mut(block.entity).unwrap();

            // Apply gravity
            let accel = v2!(0., gravity);
            block_ent.velocity += accel * dt;

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
    let typ = (inle_core::rand::rand_range(rng, 0.0..3.0) as u8).into();
    let mut sprite = make_block_sprite(env, gres, typ);
    sprite.z_index = super::Z_BLOCKS;

    let mut entity = Entity::new(sprite);
    let phys_data = Phys_Data {
        inv_mass: Cfg_Var::new("game/physics/block/mass", cfg),
        restitution: Cfg_Var::new("game/physics/restitution", cfg),
        static_friction: Cfg_Var::new("game/physics/static_friction", cfg),
        dyn_friction: Cfg_Var::new("game/physics/dyn_friction", cfg),
    };
    entity.register_to_physics(phys_world, &phys_data, GCL::Blocks, Phys_Type::Dynamic);

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
