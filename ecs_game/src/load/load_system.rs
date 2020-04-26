use crate::gameplay_system::Gameplay_System_Config;
use crate::gameplay_system::Level;
use crate::spatial::World_Chunks;
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
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::components::gfx::{C_Animated_Sprite, C_Camera2D, C_Renderable};
use ecs_engine::ecs::ecs_world::{Ecs_World, Entity};
use ecs_engine::gfx;
use ecs_engine::resources::gfx::{tex_path, Gfx_Resources};

#[cfg(debug_assertions)]
use crate::debug::entity_debug::C_Debug_Data;

#[derive(Copy, Clone, Default)]
pub struct C_Ground {
    pub neighbours: [Entity; 4],
}

pub fn level_load_sync(
    level_id: String_Id,
    engine_state: &mut Engine_State,
    game_resources: &mut Game_Resources,
    gs_cfg: Gameplay_System_Config,
) -> Level {
    let mut level = Level {
        id: level_id,
        world: Ecs_World::new(),
        cameras: vec![],
        active_camera: 0,
        chunks: World_Chunks::new(),
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
    fill_world_chunks(&mut level.chunks, &mut level.world);
    calc_terrain_colliders(&mut level.world);
    lok!(
        "Loaded level {}. N. entities = {}, n. cameras = {}",
        level_id,
        level.world.entities().len(),
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
    world.register_component::<C_Ground>();

    #[cfg(debug_assertions)]
    {
        world.register_component::<C_Debug_Data>();
    }
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
        let cam = level.world.add_component(camera, C_Camera2D::default());
        //cam.transform.set_scale(2.5, 2.5);
        cam.transform.set_position(-300., -300.);
    }
    level.cameras.push(camera);

    {
        let mut ctrl = level.world.add_component(
            camera,
            C_Controllable {
                speed: Cfg_Var::new("game/gameplay/player/player_speed", cfg),
                ..Default::default()
            },
        );
    }

    let ground = level.world.new_entity();

    {
        let rend = level.world.add_component(
            ground,
            C_Renderable {
                texture: rsrc.load_texture(&tex_path(&env, "ground.png")),
                z_index: -1,
                ..Default::default()
            },
        );
        assert!(rend.texture.is_some(), "Could not load texture!");
        let (sw, sh) = gfx::render::get_texture_size(rsrc.get_texture(rend.texture));
        rend.rect = Rect::new(0, 0, sw as i32 * 100, sh as i32 * 100);
        rsrc.get_texture_mut(rend.texture).set_repeated(true);

        level.world.add_component(ground, C_Spatial2D::default());
    }

    let ext = 20;
    let int = 5;
    let sw = 32;
    let sh = 32;
    for x in -ext..=ext {
        for y in -ext..=ext {
            if (-int..=int).contains(&x) && (-int..=int).contains(&y) {
                continue;
            }
            spawn_rock_at(level, env, rsrc, v2!((x * sw) as f32, (y * sh) as f32));
        }
    }

    let n_frames = 3;
    for i in 0..gs_cfg.n_entities_to_spawn {
        let entity = level.world.new_entity();
        let (sw, sh) = {
            let rend = level.world.add_component(
                entity,
                C_Renderable {
                    //rend.texture = rsrc.load_texture(&tex_path(&env, "yv.png"));
                    //rend.texture = rsrc.load_texture(&tex_path(&env, "plant.png"));
                    texture: rsrc.load_texture(&tex_path(&env, "jelly.png")),
                    modulate: if i == 1 {
                        colors::rgb(0, 255, 0)
                    } else {
                        colors::WHITE
                    },
                    ..Default::default()
                },
            );
            assert!(rend.texture.is_some(), "Could not load texture!");
            let (sw, sh) = gfx::render::get_texture_size(rsrc.get_texture(rend.texture));
            rend.rect = Rect::new(0, 0, sw as i32 / (n_frames as i32), sh as i32);
            (sw, sh)
        };
        if i == 1 {
            let ctr = level.world.add_component(
                entity,
                C_Controllable {
                    speed: Cfg_Var::new("game/gameplay/player/player_speed", cfg),
                    ..Default::default()
                },
            );
        }
        {
            let t = level.world.add_component(entity, C_Spatial2D::default());
            let x = rand::rand_01(rng);
            let y = rand::rand_01(rng);
            if i > 0 {
                //t.local_transform.set_position(i as f32 * 242.0, 0.);
                t.transform.set_position(x * 50., 1. * y * 50.);
                //t.local_transform.set_rotation(angle::deg(45. * i as f32));
                //t.local_transform.set_scale(2., 4.);
            }
        }
        {
            let c = level.world.add_component(
                entity,
                Collider {
                    shape: {
                        let width = (sw / n_frames) as f32;
                        let height = sh as f32;
                        Collision_Shape::Rect { width, height }
                    },
                    ..Default::default()
                },
            );
            //c.shape = Collision_Shape::Circle {
            //radius: width.max(height) * 0.5,
            //};
        }
        {
            let p = level.world.add_component(
                entity,
                C_Phys_Data {
                    inv_mass: 1.,
                    restitution: 1.0,
                    static_friction: 0.5,
                    dyn_friction: 0.3,
                },
            );
        }
        {
            let s = level.world.add_component(
                entity,
                C_Animated_Sprite {
                    n_frames,
                    frame_time: 0.12,
                    ..Default::default()
                },
            );
        }
        level
            .world
            .add_component(entity, C_Dumb_Movement::default());

        #[cfg(debug_assertions)]
        {
            level.world.add_component(entity, C_Debug_Data::default());
        }
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
    }

    //spawn_rock_at(level, env, rsrc, v2!(0., 100.), ground);
}

fn spawn_rock_at(level: &mut Level, env: &Env_Info, rsrc: &mut Gfx_Resources, pos: Vec2f) {
    let rock = level.world.new_entity();

    {
        let rend = level.world.add_component(rock, C_Renderable::default());
        rend.texture = rsrc.load_texture(&tex_path(&env, "rock.png"));
        rend.z_index = 1;
        assert!(rend.texture.is_some(), "Could not load texture!");
        let (sw, sh) = gfx::render::get_texture_size(rsrc.get_texture(rend.texture));
        let (sw, sh) = (sw as i32, sh as i32);
        rend.rect = Rect::new(0, 0, sw, sh);
    };

    {
        let t = level.world.add_component(rock, C_Spatial2D::default());
        t.transform.set_position_v(pos);
    }

    level.world.add_component(rock, C_Ground::default());

    {
        level.world.add_component(
            rock,
            C_Phys_Data {
                inv_mass: 0., // infinite mass
                restitution: 1.0,
                static_friction: 0.5,
                dyn_friction: 0.3,
            },
        );
    }
}

fn calc_terrain_colliders(world: &mut Ecs_World) {
    use ecs_engine::common::vector::Vec2i;
    use std::collections::HashMap;

    const ROCK_SIZE: f32 = 32.;
    let mut rocks_by_tile = HashMap::new();

    // for each rock ...
    foreach_entity!(world, +C_Ground, |entity| {
        let pos = world.get_component::<C_Spatial2D>(entity).unwrap().transform.position();
        let tile = Vec2i::from(pos / ROCK_SIZE);
        rocks_by_tile.insert(tile, entity);
    });

    foreach_entity!(world, +C_Ground, |entity| {
        let pos = world.get_component::<C_Spatial2D>(entity).unwrap().transform.position();
        let tile = Vec2i::from(pos / ROCK_SIZE);

        let up = tile - v2!(0, 1);
        let down = tile + v2!(0, 1);
        let left = tile - v2!(1, 0);
        let right = tile + v2!(1, 0);

        let mut neighs = 0;
        let e_up = rocks_by_tile.get(&up);
        let e_right = rocks_by_tile.get(&right);
        let e_left = rocks_by_tile.get(&left);
        let e_down = rocks_by_tile.get(&down);

        let ground = world.get_component_mut::<C_Ground>(entity).unwrap();
        // Note: order must be the same as Square_Directions!
        for (i, &dir) in [e_up, e_right, e_down, e_left].iter().enumerate() {
            if let Some(e) = dir {
                neighs += 1;
                ground.neighbours[i] = *e;
            }
        }

        if neighs < 4 {
            world.add_component(entity, Collider {
                shape: Collision_Shape::Rect {
                    width: ROCK_SIZE,
                    height: ROCK_SIZE
                },
                is_static: true,
                ..Default::default()
            });
        }
    });
}

fn fill_world_chunks(chunks: &mut World_Chunks, world: &mut Ecs_World) {
    foreach_entity!(world, +C_Spatial2D, |entity| {
        let spatial = world.get_component_mut::<C_Spatial2D>(entity).unwrap();

        // @Cleanup @Soundness: this is not the right place to set this value! It should
        // be done after updating the scene tree for the first time (and use the global transform)!
        spatial.frame_starting_pos = spatial.transform.position();

        chunks.add_entity(entity, spatial.transform.position());
    });
}
