use crate::gameplay_system::Gameplay_System_Config;
use crate::gameplay_system::Level;
use crate::systems::controllable_system::C_Controllable;
use crate::systems::dumb_movement_system::C_Dumb_Movement;
use crate::Game_Resources;
use ecs_engine::cfg::{self, Cfg_Var};
use ecs_engine::collisions::collider::{C_Phys_Data, Collider, Collision_Shape};
use ecs_engine::common::colors;
use ecs_engine::common::rect::Rect;
use ecs_engine::common::stringid::String_Id;
use ecs_engine::common::transform::Transform2D;
use ecs_engine::common::vector::Vec2f;
use ecs_engine::core::app::Engine_State;
use ecs_engine::core::env::Env_Info;
use ecs_engine::core::rand;
use ecs_engine::core::scene_tree::Scene_Tree;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::components::gfx::{C_Animated_Sprite, C_Camera2D, C_Renderable};
use ecs_engine::ecs::ecs_world::{Ecs_World, Entity};
use ecs_engine::gfx;
use ecs_engine::resources::gfx::{tex_path, Gfx_Resources};

pub fn level_load_sync(
    level_id: String_Id,
    engine_state: &mut Engine_State,
    game_resources: &mut Game_Resources,
    gs_cfg: Gameplay_System_Config,
) -> Level {
    let mut level = Level {
        id: level_id,
        world: Ecs_World::new(),
        entities: vec![],
        cameras: vec![],
        active_camera: 0,
        scene_tree: Scene_Tree::new(),
    };

    linfo!("Loading level {} ...", level_id);
    register_all_components(&mut level.world);
    init_demo_entities(
        &mut game_resources.gfx,
        &engine_state.env,
        &mut engine_state.rng,
        &engine_state.config,
        &mut level,
        gs_cfg,
    );
    lok!(
        "Loaded level {}. N. entities = {}, n. cameras = {}",
        level_id,
        level.entities.len(),
        level.cameras.len()
    );

    level
}

fn register_all_components(world: &mut Ecs_World) {
    world.register_component::<C_Spatial2D>();
    world.register_component::<Transform2D>();
    world.register_component::<C_Camera2D>();
    world.register_component::<C_Renderable>();
    world.register_component::<C_Animated_Sprite>();
    world.register_component::<C_Controllable>();
    world.register_component::<Collider>();
    world.register_component::<C_Dumb_Movement>();
    world.register_component::<C_Phys_Data>();
}

// @Temporary
fn init_demo_entities(
    rsrc: &mut Gfx_Resources,
    env: &Env_Info,
    rng: &mut rand::Default_Rng,
    cfg: &cfg::Config,
    level: &mut Level,
    gs_cfg: Gameplay_System_Config,
) {
    #![allow(warnings)]
    use ecs_engine::common::angle;

    let camera = level.world.new_entity();
    {
        let cam = level.world.add_component::<C_Camera2D>(camera);
        //cam.transform.set_scale(2.5, 2.5);
        cam.transform.set_position(-300., -300.);
    }
    level.cameras.push(camera);

    {
        let mut ctrl = level.world.add_component::<C_Controllable>(camera);
        ctrl.speed = Cfg_Var::new("game/gameplay/player/player_speed", cfg);
    }

    let mut prev_entity: Option<Entity> = None;
    let ground = level.world.new_entity();

    {
        let rend = level.world.add_component::<C_Renderable>(ground);
        rend.texture = rsrc.load_texture(&tex_path(&env, "ground.png"));
        rend.z_index = -1;
        assert!(rend.texture.is_some(), "Could not load texture!");
        let (sw, sh) = gfx::render::get_texture_size(rsrc.get_texture(rend.texture));
        rsrc.get_texture_mut(rend.texture).set_repeated(true);
        rend.rect = Rect::new(0, 0, sw as i32 * 100, sh as i32 * 100);

        let t = level.world.add_component::<C_Spatial2D>(ground);
        level.scene_tree.add(ground, None, &t.local_transform);
    }

    let ext = 4;
    let int = 2;
    let sw = 32;
    let sh = 32;
    for x in -ext..=ext {
        for y in -ext..=ext {
            if (-int..=int).contains(&x) && (-int..=int).contains(&y) {
                continue;
            }
            spawn_rock_at(
                level,
                env,
                rsrc,
                v2!((x * sw) as f32, (y * sh) as f32),
                ground,
            );
        }
    }

    let n_frames = 3;
    for i in 0..gs_cfg.n_entities_to_spawn {
        let entity = level.world.new_entity();
        let (sw, sh) = {
            let rend = level.world.add_component::<C_Renderable>(entity);
            //rend.texture = rsrc.load_texture(&tex_path(&env, "yv.png"));
            //rend.texture = rsrc.load_texture(&tex_path(&env, "plant.png"));
            rend.texture = rsrc.load_texture(&tex_path(&env, "jelly.png"));
            assert!(rend.texture.is_some(), "Could not load texture!");
            rend.modulate = if i == 1 {
                colors::rgb(0, 255, 0)
            } else {
                colors::WHITE
            };
            let (sw, sh) = gfx::render::get_texture_size(rsrc.get_texture(rend.texture));
            rend.rect = Rect::new(0, 0, sw as i32 / (n_frames as i32), sh as i32);
            (sw, sh)
        };
        if i == 1 {
            let ctr = level.world.add_component::<C_Controllable>(entity);
            ctr.speed = Cfg_Var::new("game/gameplay/player/player_speed", cfg);
        }
        {
            let t = level.world.add_component::<C_Spatial2D>(entity);
            let x = rand::rand_01(rng);
            let y = rand::rand_01(rng);
            if i > 0 {
                //t.local_transform.set_position(i as f32 * 242.0, 0.);
                t.local_transform.set_position(x * 50., 1. * y * 50.);
                //t.local_transform.set_rotation(angle::deg(45. * i as f32));
                //t.local_transform.set_scale(2., 4.);
            }
            level
                .scene_tree
                .add(entity, Some(ground), &t.local_transform);
        }
        {
            let c = level.world.add_component::<Collider>(entity);
            let width = (sw / n_frames) as f32;
            let height = sh as f32;
            c.shape = Collision_Shape::Rect { width, height };
            //c.shape = Collision_Shape::Circle {
            //radius: width.max(height) * 0.5,
            //};
        }
        {
            let p = level.world.add_component::<C_Phys_Data>(entity);
            p.inv_mass = 1.;
            p.restitution = 1.0;
            p.static_friction = 0.5;
            p.dyn_friction = 0.3;
        }
        {
            let s = level.world.add_component::<C_Animated_Sprite>(entity);
            s.n_frames = n_frames;
            s.frame_time = 0.12;
        }
        level.world.add_component::<C_Dumb_Movement>(entity);
        prev_entity = Some(entity);
        //{
        //    let mut t = level.world.add_component::<C_Spatial2D>(entity);
        //    t.transform.set_origin(sw as f32 * 0.5, sh as f32 * 0.5);
        //    t.transform
        //        .set_position(n as f32 * (i % n) as f32, n as f32 * (i / n) as f32);
        //}
        //{
        //let mut ctrl = level.world.add_component::<C_Controllable>(entity);
        //ctrl.speed = cfg.get_var_float_or("gameplay/player/player_speed", 300.0);
        //}
        level.entities.push(entity);
    }

    spawn_rock_at(level, env, rsrc, v2!(0., 100.), ground);
}

fn spawn_rock_at(
    level: &mut Level,
    env: &Env_Info,
    rsrc: &mut Gfx_Resources,
    pos: Vec2f,
    ground: Entity,
) {
    let rock = level.world.new_entity();

    let (sw, sh) = {
        let rend = level.world.add_component::<C_Renderable>(rock);
        rend.texture = rsrc.load_texture(&tex_path(&env, "rock.png"));
        rend.z_index = 1;
        assert!(rend.texture.is_some(), "Could not load texture!");
        let (sw, sh) = gfx::render::get_texture_size(rsrc.get_texture(rend.texture));
        let (sw, sh) = (sw as i32, sh as i32);
        rend.rect = Rect::new(0, 0, sw, sh);
        (sw, sh)
    };

    {
        let t = level.world.add_component::<C_Spatial2D>(rock);
        t.local_transform.set_position_v(pos);
        level.scene_tree.add(rock, Some(ground), &t.local_transform);
    }

    {
        let c = level.world.add_component::<Collider>(rock);
        c.shape = Collision_Shape::Rect {
            width: sw as f32,
            height: sh as f32,
        };
    }
    {
        let p = level.world.add_component::<C_Phys_Data>(rock);
        p.inv_mass = 0.; // infinite mass
        p.restitution = 1.0;
        p.static_friction = 0.5;
        p.dyn_friction = 0.3;
    }
    level.entities.push(rock);
}
