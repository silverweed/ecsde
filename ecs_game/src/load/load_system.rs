use crate::collisions::Game_Collision_Layer;
use crate::entities;
use crate::gameplay_system::Gameplay_System_Config;
use crate::gfx::multi_sprite_animation_system::C_Multi_Renderable_Animation;
use crate::gfx::shaders::*;
use crate::levels::Level;
use crate::spatial::World_Chunks;
use crate::systems::controllable_system::C_Controllable;
use crate::systems::dumb_movement_system::C_Dumb_Movement;
use crate::systems::ground_collision_calculation_system::C_Ground;
use crate::systems::pixel_collision_system::C_Texture_Collider;
use crate::Game_Resources;
use ecs_engine::cfg::{self, Cfg_Var};
use ecs_engine::collisions::collider::{C_Phys_Data, Collider, Collision_Shape};
use ecs_engine::common::angle::rad;
use ecs_engine::common::colors;
use ecs_engine::common::rect::Rect;
use ecs_engine::common::stringid::String_Id;
use ecs_engine::common::transform::Transform2D;
use ecs_engine::core::app::Engine_State;
use ecs_engine::core::env::Env_Info;
use ecs_engine::core::rand;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::components::gfx::{
    C_Animated_Sprite, C_Camera2D, C_Multi_Renderable, C_Renderable, Material,
};
use ecs_engine::ecs::ecs_world::Ecs_World;
use ecs_engine::gfx;
use ecs_engine::gfx::light::{Lights, Point_Light};
use ecs_engine::resources::gfx::{tex_path, Gfx_Resources, Shader_Cache};

#[cfg(debug_assertions)]
use crate::debug::entity_debug::C_Debug_Data;

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
        lights: Lights::default(),
    };

    linfo!("Loading level {} ...", level_id);
    register_all_components(&mut level.world);
    init_demo_entities(
        &mut game_resources.gfx,
        &mut engine_state.shader_cache,
        &engine_state.env,
        &mut engine_state.rng,
        &engine_state.config,
        &mut level,
        gs_cfg,
    );
    init_demo_lights(&mut level.lights);
    calc_terrain_colliders(&mut level.world);
    fill_world_chunks(&mut level.chunks, &mut level.world);
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
    world.register_component::<C_Texture_Collider>();
    world.register_component::<C_Multi_Renderable>();
    world.register_component::<C_Multi_Renderable_Animation>();

    #[cfg(debug_assertions)]
    {
        world.register_component::<C_Debug_Data>();
    }
}

// @Temporary
fn init_demo_lights(lights: &mut Lights) {
    lights.ambient_light.color = colors::rgb(200, 140, 180);
    lights.ambient_light.intensity = 0.2;

    let light = Point_Light {
        position: v2!(0., -100.),
        radius: 350.,
        attenuation: 0.0,
        color: colors::WHITE,
        intensity: 2.0,
    };
    lights.add_point_light(light);
}

// @Temporary
fn init_demo_entities(
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    rng: &mut rand::Default_Rng,
    cfg: &cfg::Config,
    level: &mut Level,
    gs_cfg: Gameplay_System_Config,
) {
    #![allow(warnings)]
    use ecs_engine::common::angle;
    use ecs_engine::resources::gfx::shader_path;

    let sprite_normal_shader =
        shader_cache.load_shader(&shader_path(&env, SHD_SPRITE_WITH_NORMALS));
    let sprite_flat_shader = shader_cache.load_shader(&shader_path(&env, SHD_SPRITE_FLAT));
    let terrain_shader = shader_cache.load_shader(&shader_path(&env, SHD_TERRAIN));
    let sprite_unlit_shader = shader_cache.load_shader(&shader_path(&env, SHD_SPRITE_UNLIT));

    let camera = level.world.new_entity();
    {
        let cam = level.world.add_component(camera, C_Camera2D::default());
        cam.transform.set_scale(0.2, 0.2);
        cam.transform.set_position(-120., -75.);
    }
    level.cameras.push(camera);

    {
        let mut ctrl = level.world.add_component(
            camera,
            C_Controllable {
                speed: Cfg_Var::new("game/gameplay/camera_speed", cfg),
                ..Default::default()
            },
        );
    }

    {
        let ground = level.world.new_entity();
        let rend = level.world.add_component(
            ground,
            C_Renderable {
                material: Material {
                    texture: gres.load_texture(&tex_path(&env, "ground.png")),
                    shader: sprite_flat_shader,
                    ..Default::default()
                },
                z_index: -1,
                ..Default::default()
            },
        );
        assert!(rend.material.texture.is_some(), "Could not load texture!");
        let (sw, sh) = gfx::render::get_texture_size(gres.get_texture(rend.material.texture));
        rend.rect = Rect::new(0, 0, sw as i32 * 100, sh as i32 * 100);
        gfx::render::set_texture_repeated(gres.get_texture_mut(rend.material.texture), true);

        level.world.add_component(ground, C_Spatial2D::default());
    }

    let ext = 0;
    let int = 5;
    let sw = 32;
    let sh = 32;
    for x in -ext..=ext {
        for y in -ext..=ext {
            if (-int..=int).contains(&x) && (-int..=int).contains(&y) {
                continue;
            }
            entities::create_rock(
                &mut level.world,
                gres,
                env,
                &Transform2D::from_pos(v2!((x * sw) as f32, (y * sh) as f32)),
            );
        }
    }

    // Spawn terrain
    {
        let gnd = level.world.new_entity();

        {
            let t = level.world.add_component(gnd, C_Spatial2D::default());
            t.transform.set_position(0., 600.);
        }

        let rend = level.world.add_component(
            gnd,
            C_Renderable {
                material: Material {
                    texture: gres.load_texture(&tex_path(&env, "ground3.png")),
                    shader: terrain_shader,
                    shininess: Material::encode_shininess(0.2),
                    ..Default::default()
                },
                z_index: 1,
                ..Default::default()
            },
        );
        assert!(rend.material.texture.is_some(), "Could not load texture!");
        let (sw, sh) = gfx::render::get_texture_size(gres.get_texture(rend.material.texture));
        rend.rect = Rect::new(0, 0, sw as i32 * 1, sh as i32 * 1);
        //gres.get_texture_mut(rend.texture).set_repeated(true);
        let texture = rend.material.texture;

        level.world.add_component(
            gnd,
            C_Texture_Collider {
                texture,
                layer: Game_Collision_Layer::Ground as _,
            },
        );
    }

    // Sky
    {
        let sky = level.world.new_entity();

        {
            let t = level.world.add_component(sky, C_Spatial2D::default());
            t.transform.set_position(0., -370.);
        }

        let rend = level.world.add_component(
            sky,
            C_Renderable {
                material: Material {
                    texture: gres.load_texture(&tex_path(&env, "sky.png")),
                    shader: sprite_unlit_shader,
                    ..Default::default()
                },
                z_index: 0,
                ..Default::default()
            },
        );
        assert!(rend.material.texture.is_some(), "Could not load texture!");
        let (sw, sh) = gfx::render::get_texture_size(gres.get_texture(rend.material.texture));
        rend.rect = Rect::new(0, 0, sw as i32 * 1, sh as i32 * 1);
        //gres.get_texture_mut(rend.texture).set_repeated(true);
        let texture = rend.material.texture;

        level.world.add_component(
            sky,
            Collider {
                shape: Collision_Shape::Rect {
                    width: sw as f32,
                    height: sh as f32,
                },
                layer: Game_Collision_Layer::Sky as _,
                ..Default::default()
            },
        );
        level.world.add_component(
            sky,
            C_Phys_Data {
                inv_mass: 0.,
                ..Default::default()
            },
        );
    }

    for i in 0..gs_cfg.n_entities_to_spawn {
        let x = rand::rand_01(rng);
        let y = rand::rand_01(rng);
        let pos = if i > 0 {
            v2!(x * 50., 1. * y * 50.)
        } else {
            v2!(20., 20.)
        };

        entities::create_jelly(
            &mut level.world,
            gres,
            shader_cache,
            env,
            cfg,
            &Transform2D::from_pos(pos),
            i == 0,
        );
    }

    entities::create_drill(
        &mut level.world,
        gres,
        shader_cache,
        env,
        cfg,
        &Transform2D::from_pos_rot_scale(v2!(10., 10.), rad(0.), v2!(0.2, 0.2)),
    );
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
                layer: Game_Collision_Layer::Ground as _,
                is_static: true,
                ..Default::default()
            });
        }
    });
}

fn fill_world_chunks(chunks: &mut World_Chunks, world: &mut Ecs_World) {
    foreach_entity!(world, +C_Spatial2D, +Collider, |entity| {
        let spatial = world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        let pos = spatial.transform.position();
        spatial.frame_starting_pos = pos;
        let collider = world.get_component::<Collider>(entity).unwrap();
        chunks.add_entity(entity, pos, collider.shape.extent());
    });
}
