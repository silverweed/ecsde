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
use inle_cfg::Config;
use inle_math::vector::{Vec2f, Vec2u};
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
use inle_ecs::ecs_world::{Ecs_World, Entity};
use inle_gfx::components::{C_Animated_Sprite, C_Camera2D, C_Multi_Renderable, C_Renderable};
use inle_gfx::light::{Light_Command, Lights, Point_Light, Rect_Light};
use inle_math::rect::Rect;
use inle_math::transform::Transform2D;
use inle_physics::collider::C_Collider;
use inle_physics::phys_world::Physics_World;
use inle_resources::gfx::{Gfx_Resources, Shader_Cache};

struct Room_Grid {
    size: Vec2u, // number of rooms in both directions
    spacing: f32,
    room_size: Vec2f,
    first_room_center: Vec2f,
}

pub fn generate_random_level(
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    rng: &mut rand::Default_Rng,
    cfg: &Config,
    level: &mut Level,
    gs_cfg: Gameplay_System_Config,
) {
    #![allow(warnings)]
    use inle_math::angle;
    use inle_resources::gfx::shader_path;

    let camera = create_camera(level, cfg);

    entities::create_background(&mut level.world, gres, shader_cache, env, cfg);

    const GRID: Room_Grid = Room_Grid {
        size: v2!(3, 3),
        spacing: 32.0,
        room_size: v2!(600., 400.),
        first_room_center: v2!(0., 0.),
    };

    create_room_grid(GRID, level, gres, shader_cache, env, cfg);

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
}

fn create_camera(level: &mut Level, cfg: &Config) -> Entity {
    let camera = level.world.new_entity();
    {
        let cam = level.world.add_component(camera, C_Camera2D::default());
        let scale = Cfg_Var::<f32>::new("game/camera/initial_scale", cfg).read(cfg);
        cam.transform.set_scale(scale, scale);
    }
    {
        let mut ctrl = level.world.add_component(
            camera,
            C_Controllable {
                speed: Cfg_Var::new("game/camera/speed", cfg),
                ..Default::default()
            },
        );
    }

    level.cameras.push(camera);

    camera
}

fn create_room_grid(
    grid: Room_Grid,
    level: &mut Level,
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    cfg: &Config,
) {
    for x in 0..grid.size.x {
        for y in 0..grid.size.y {
            create_room(
                grid.first_room_center + grid.room_size * v2!(x as f32, y as f32),
                grid.room_size,
                grid.spacing,
                level,
                gres,
                shader_cache,
                env,
                cfg,
            );
        }
    }
}

fn create_room(
    center: Vec2f,
    room_halfsize: Vec2f,
    wall_thickness: f32,
    level: &mut Level,
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    cfg: &Config,
) {
    use entities::create_wall;

    let world = &mut level.world;
    let phys_world = &mut level.phys_world;

    // bot
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(center + v2!(0.0, room_halfsize.y)),
        v2!((room_halfsize.x + wall_thickness) * 2.0, wall_thickness),
        cfg,
    );
    // top
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(center + v2!(0.0, -room_halfsize.y)),
        v2!((room_halfsize.x + wall_thickness) * 2.0, wall_thickness),
        cfg,
    );
    // left
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(center + v2!(-room_halfsize.x, 0.0)),
        v2!(wall_thickness, (room_halfsize.y + wall_thickness) * 2.0),
        cfg,
    );
    // right
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(center + v2!(room_halfsize.x, 0.0)),
        v2!(wall_thickness, (room_halfsize.y + wall_thickness) * 2.0),
        cfg,
    );

    create_room_lights(center, &mut level.lights, cfg);
}

fn create_room_lights(room_center: Vec2f, lights: &mut Lights, cfg: &inle_cfg::Config) {
    // -------------------------------------------
    // Corner lights
    // -------------------------------------------
    lights.queue_command(Light_Command::Add_Point_Light(Point_Light {
        position: room_center + v2!(-280., -180.),
        radius: 250.,
        attenuation: 1.0,
        color: colors::YELLOW,
        intensity: 1.0,
    }));

    lights.queue_command(Light_Command::Add_Point_Light(Point_Light {
        position: room_center + v2!(280., -180.),
        radius: 250.,
        attenuation: 1.0,
        color: colors::YELLOW,
        intensity: 1.0,
    }));

    lights.queue_command(Light_Command::Add_Point_Light(Point_Light {
        position: room_center + v2!(-280., 180.),
        radius: 250.,
        attenuation: 1.0,
        color: colors::YELLOW,
        intensity: 1.0,
    }));

    lights.queue_command(Light_Command::Add_Point_Light(Point_Light {
        position: room_center + v2!(280., 180.),
        radius: 250.,
        attenuation: 1.0,
        color: colors::YELLOW,
        intensity: 1.0,
    }));

    // -------------------------------------------

    lights.queue_command(Light_Command::Add_Point_Light(Point_Light {
        position: room_center + v2!(0., 0.),
        radius: 350.,
        attenuation: 0.5,
        color: colors::DARK_ORANGE,
        intensity: 0.5,
    }));

    lights.queue_command(Light_Command::Add_Rect_Light(Rect_Light {
        rect: Rect::new(-300., -199., 600., 1.) + room_center,
        radius: 50.,
        attenuation: 1.0,
        color: colors::DARK_ORANGE,
        intensity: 0.5,
    }));

    lights.queue_command(Light_Command::Add_Rect_Light(Rect_Light {
        rect: Rect::new(-300., 199., 600., 1.) + room_center,
        radius: 50.,
        attenuation: 1.0,
        color: colors::DARK_ORANGE,
        intensity: 0.5,
    }));

    lights.queue_command(Light_Command::Add_Rect_Light(Rect_Light {
        rect: Rect::new(-299., -200., 1., 400.) + room_center,
        radius: 50.,
        attenuation: 1.0,
        color: colors::DARK_ORANGE,
        intensity: 0.5,
    }));

    lights.queue_command(Light_Command::Add_Rect_Light(Rect_Light {
        rect: Rect::new(299., -200., 1., 400.) + room_center,
        radius: 50.,
        attenuation: 1.0,
        color: colors::DARK_ORANGE,
        intensity: 0.5,
    }));

    //let light = Point_Light {
    //position: v2!(0., 0.),
    //radius: 150.,
    //attenuation: 0.0,
    //color: colors::GREEN,
    //intensity: 1.0,
    //};
    //lights.add_point_light(light);
}
