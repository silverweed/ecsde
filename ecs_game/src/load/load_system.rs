// @Temporary
#![allow(warnings)]

use crate::entities;
use crate::gameplay_system::Gameplay_System_Config;
use crate::gfx::multi_sprite_animation_system::C_Multi_Renderable_Animation;
use crate::levels::Level;
use crate::spatial::World_Chunks;
use crate::systems::camera_system::{C_Camera_Follow, Camera_Follow_Target};
use crate::systems::controllable_system::C_Controllable;
use crate::systems::gravity_system::C_Gravity;
//use crate::systems::ground_collision_calculation_system::C_Ground;
use crate::systems::ground_detection_system::C_Ground_Detection;
use crate::Game_Resources;
use inle_app::app::Engine_State;
use inle_cfg::{self, Cfg_Var};
use inle_common::colors;
use inle_common::stringid::String_Id;
use inle_core::env::Env_Info;
use inle_core::rand;
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::Ecs_World;
use inle_gfx::components::{C_Animated_Sprite, C_Camera2D, C_Multi_Renderable, C_Renderable};
use inle_gfx::light::{Ambient_Light, Light_Command, Lights, Point_Light, Rect_Light};
use inle_math::rect::Rect;
use inle_math::transform::Transform2D;
use inle_physics::collider::C_Collider;
use inle_physics::phys_world::Physics_World;
use inle_resources::gfx::{Gfx_Resources, Shader_Cache};

#[cfg(debug_assertions)]
use {
    crate::debug::entity_debug::C_Debug_Data,
    crate::debug::systems::position_history_system::C_Position_History,
};

pub fn level_load_sync(
    level_id: String_Id,
    engine_state: &mut Engine_State,
    game_resources: &mut Game_Resources,
    gs_cfg: Gameplay_System_Config,
    cvars: &crate::game_state::CVars,
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
    init_demo_entities(
        &mut game_resources.gfx,
        &mut game_resources.shader_cache,
        &engine_state.env,
        &mut engine_state.rng,
        &engine_state.config,
        &mut level,
        gs_cfg,
    );
    init_demo_lights(&mut level.lights, &engine_state.config, cvars);
    fill_world_chunks(&mut level.chunks, &mut level.world, &level.phys_world);
    lok!(
        "Loaded level {}. N. entities = {}, n. cameras = {}",
        level_id,
        level.world.entities().len(),
        level.cameras.len()
    );

    level
}

// @Temporary
fn init_demo_lights(lights: &mut Lights, cfg: &inle_cfg::Config, cvars: &crate::game_state::CVars) {
    let amb_intensity = cvars.ambient_intensity.read(cfg);
    let amb_color = colors::color_from_hex(cvars.ambient_color.read(cfg));
    let ambient_light = Ambient_Light {
        color: amb_color,
        intensity: amb_intensity,
    };
    lights.queue_command(Light_Command::Change_Ambient_Light(ambient_light));

    //let light = Rect_Light {
    //    rect: Rect::new(-37., -140., 90., 30.),
    //    radius: 300.,
    //    attenuation: 2.5,
    //    color: colors::WHITE,
    //    intensity: 2.5,
    //};
    //lights.add_rect_light(light);
}

// @Temporary
fn init_demo_entities(
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    rng: &mut rand::Default_Rng,
    cfg: &inle_cfg::Config,
    level: &mut Level,
    gs_cfg: Gameplay_System_Config,
) {
    super::proc_gen::generate_random_level(gres, shader_cache, env, rng, cfg, level, gs_cfg);
    /*
        #![allow(warnings)]
        use inle_math::angle;
        use inle_resources::gfx::shader_path;

        let camera = level.world.new_entity();
        {
            let cam = level.world.add_component(camera, C_Camera2D::default());
            let scale = Cfg_Var::<f32>::new("game/camera/initial_scale", cfg).read(cfg);
            cam.transform.set_scale(scale, scale);
            cam.transform.set_position(-120., -75.);
        }
        level.cameras.push(camera);

        {
            let mut ctrl = level.world.add_component(
                camera,
                C_Controllable {
                    speed: Cfg_Var::new("game/camera/speed", cfg),
                    ..Default::default()
                },
            );
        }

        entities::create_background(&mut level.world, gres, shader_cache, env, cfg);

        //entities::create_terrain(&mut level.world, gres, shader_cache, env, cfg);
        entities::create_room(
            &mut level.world,
            &mut level.phys_world,
            gres,
            shader_cache,
            env,
            cfg,
        );

        //entities::create_sky(
        //    &mut level.world,
        //    &mut level.phys_world,
        //    gres,
        //    shader_cache,
        //    env,
        //    cfg,
        //);

        for i in 0..gs_cfg.n_entities_to_spawn {
            let x = rand::rand_01(rng);
            let y = rand::rand_01(rng);
            let pos = if i > 0 {
                v2!(x * 50., 1. * y * 50.)
            } else {
                v2!(20., 20.)
            };

            let player = entities::create_jelly(
                &mut level.world,
                &mut level.phys_world,
                gres,
                shader_cache,
                env,
                cfg,
                &Transform2D::from_pos(pos),
                i == 0,
            );

            if i == 0 {
                level.world.add_component(
                    camera,
                    C_Camera_Follow {
                        target: Camera_Follow_Target::Entity(player),
                        lerp_factor: Cfg_Var::new("game/camera/lerp_factor", cfg),
                    },
                );
            }
        }

        //entities::create_drill(
        //    &mut level.world,
        //    &mut level.phys_world,
        //    gres,
        //    shader_cache,
        //    env,
        //    cfg,
        //    &Transform2D::from_pos_rot_scale(v2!(10., 10.), rad(0.), v2!(0.2, 0.2)),
        //);
    */
}

fn fill_world_chunks(chunks: &mut World_Chunks, world: &mut Ecs_World, phys_world: &Physics_World) {
    foreach_entity_new!(world,
        read: C_Collider;
        write: C_Spatial2D;
        |entity, (collider,): (&C_Collider,), (spatial,): (&mut C_Spatial2D,)| {
        let pos = spatial.transform.position();
        spatial.frame_starting_pos = pos;
        let body_handle = collider.handle;
        for (collider, cld_handle) in phys_world.get_all_colliders_with_handles(body_handle) {
            chunks.add_collider(cld_handle, pos, collider.shape.extent());
        }
    });
}
