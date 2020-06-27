use crate::entities;
use crate::gameplay_system::Gameplay_System_Config;
use crate::gfx::multi_sprite_animation_system::C_Multi_Renderable_Animation;
use crate::gfx::shaders::*;
use crate::levels::Level;
use crate::spatial::World_Chunks;
use crate::systems::controllable_system::C_Controllable;
use crate::systems::dumb_movement_system::C_Dumb_Movement;
use crate::systems::entity_preview_system::C_Entity_Preview;
use crate::systems::ground_collision_calculation_system::C_Ground;
use crate::systems::pixel_collision_system::C_Texture_Collider;
use crate::Game_Resources;
use ecs_engine::cfg::{self, Cfg_Var};
use ecs_engine::collisions::collider::C_Collider;
use ecs_engine::collisions::phys_world::Physics_World;
use ecs_engine::common::angle::rad;
use ecs_engine::common::colors;
use ecs_engine::common::stringid::String_Id;
use ecs_engine::common::transform::Transform2D;
use ecs_engine::core::app::Engine_State;
use ecs_engine::core::env::Env_Info;
use ecs_engine::core::rand;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::components::gfx::{
    C_Animated_Sprite, C_Camera2D, C_Multi_Renderable, C_Renderable,
};
use ecs_engine::ecs::ecs_world::Ecs_World;
use ecs_engine::gfx::light::{Lights, Point_Light};
use ecs_engine::resources::gfx::{Gfx_Resources, Shader_Cache};

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
        phys_world: Physics_World::new(),
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
    fill_world_chunks(&mut level.chunks, &mut level.world, &mut level.phys_world);
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
    world.register_component::<C_Dumb_Movement>();
    world.register_component::<C_Collider>();
    world.register_component::<C_Ground>();
    world.register_component::<C_Texture_Collider>();
    world.register_component::<C_Multi_Renderable>();
    world.register_component::<C_Multi_Renderable_Animation>();
    world.register_component::<C_Entity_Preview>();

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

    entities::create_background(&mut level.world, gres, shader_cache, env, cfg);

    entities::create_terrain(&mut level.world, gres, shader_cache, env, cfg);

    entities::create_sky(
        &mut level.world,
        &mut level.phys_world,
        gres,
        shader_cache,
        env,
        cfg,
    );

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
            &mut level.phys_world,
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
        &mut level.phys_world,
        gres,
        shader_cache,
        env,
        cfg,
        &Transform2D::from_pos_rot_scale(v2!(10., 10.), rad(0.), v2!(0.2, 0.2)),
    );
}

fn fill_world_chunks(chunks: &mut World_Chunks, world: &mut Ecs_World, phys_world: &Physics_World) {
    foreach_entity!(world, +C_Spatial2D, +C_Collider, |entity| {
        let spatial = world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        let pos = spatial.transform.position();
        spatial.frame_starting_pos = pos;
        let body_handle = world.get_component::<C_Collider>(entity).unwrap().handle;
        let phys_body = phys_world.get_physics_body(body_handle).unwrap();
        for cld in phys_body.all_colliders() {
            let collider = phys_world.get_collider(cld).unwrap();
            chunks.add_entity(entity, pos, collider.shape.extent());
        }
    });
}
