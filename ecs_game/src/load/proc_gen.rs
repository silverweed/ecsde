// @Temporary
#![allow(warnings)]

use crate::entities;
use crate::gameplay_system::Gameplay_System_Config;
use crate::gfx::multi_sprite_animation_system::C_Multi_Renderable_Animation;
use crate::levels::{Level, Spawn_Point};
use crate::spatial::World_Chunks;
use crate::systems::camera_system::{C_Camera_Follow, Camera_Follow_Target};
use crate::systems::controllable_system::C_Controllable;
use crate::systems::gravity_system::C_Gravity;
use inle_cfg::Config;
use inle_math::vector::{Vec2f, Vec2u};
//use crate::systems::ground_collision_calculation_system::C_Ground;
use crate::collisions::Game_Collision_Layer;
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
use inle_physics::collider::{C_Collider, Collider, Collision_Shape};
use inle_physics::phys_world::Physics_World;
use inle_resources::gfx::{Gfx_Resources, Shader_Cache};

struct Room_Grid {
    size: Vec2u, // number of rooms in both directions
    spacing: f32,
    room_halfsize: Vec2f,
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
    use inle_math::angle;
    use inle_resources::gfx::shader_path;

    entities::create_background(&mut level.world, gres, shader_cache, env, cfg);

    const GRID: Room_Grid = Room_Grid {
        size: v2!(3, 3),
        spacing: 32.0,
        room_halfsize: v2!(300., 200.),
        first_room_center: v2!(0., 0.),
    };

    create_room_grid(GRID, level, gres, shader_cache, env, cfg);

    level.data.player_spawn_point = Spawn_Point {
        position: v2!(20., 20.),
    };

    level
        .data
        .ai_spawn_points
        .reserve(gs_cfg.n_entities_to_spawn - 1);
    for i in 0..gs_cfg.n_entities_to_spawn - 1 {
        let x = rand::rand_01(rng);
        let y = rand::rand_01(rng);
        let pos = v2!(x * 500., 1. * y * 500.);
        level
            .data
            .ai_spawn_points
            .push(Spawn_Point { position: pos });
    }
}

pub fn generate_enemies(
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    cfg: &Config,
    level: &mut Level,
) {
    // This is @Temporary
    for spawn_point in &level.data.ai_spawn_points {
        create_enemy(
            gres,
            shader_cache,
            env,
            cfg,
            &mut level.world,
            &mut level.phys_world,
            spawn_point.position,
        );
    }
}

fn create_enemy(
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    cfg: &Config,
    world: &mut Ecs_World,
    phys_world: &mut Physics_World,
    position: Vec2f,
) {
    use crate::systems::ai::test_ai_system::C_Test_Ai;

    let enemy = entities::create_jelly(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        cfg,
        &Transform2D::from_pos(position),
        false,
    );

    let phys_body_handle = world.get_component::<C_Collider>(enemy).unwrap().handle;
    let right_cld = Collider {
        shape: Collision_Shape::Rect {
            width: 4.0,
            height: 6.0,
        },
        offset: v2!(6., 8.),
        is_static: false,
        layer: Game_Collision_Layer::Ground_Check as _,
        entity: enemy,
        ..Default::default()
    };
    let right_cld_handle = phys_world.add_collider(right_cld.clone());
    let left_cld = Collider {
        offset: v2!(-6., 8.),
        ..right_cld
    };
    let left_cld_handle = phys_world.add_collider(left_cld);

    let phys_body = phys_world.get_physics_body_mut(phys_body_handle).unwrap();

    phys_body.trigger_colliders.push(right_cld_handle);
    phys_body.trigger_colliders.push(left_cld_handle);

    world.add_component(enemy, C_Test_Ai::new(left_cld_handle, right_cld_handle));
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
                grid.first_room_center + 2.0 * grid.room_halfsize * v2!(x as f32, y as f32),
                grid.room_halfsize,
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
        &Transform2D::from_pos(center + v2!(-room_halfsize.x - wall_thickness, room_halfsize.y)),
        v2!(room_halfsize.x + wall_thickness, wall_thickness),
        cfg,
    );
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(center + v2!(2.0 * wall_thickness, room_halfsize.y)),
        v2!(room_halfsize.x - wall_thickness, wall_thickness),
        cfg,
    );
    // top
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(center + v2!(-room_halfsize.x, -room_halfsize.y - wall_thickness)),
        v2!(room_halfsize.x, wall_thickness),
        cfg,
    );
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(
            center + v2!(2.0 * wall_thickness, -room_halfsize.y - wall_thickness),
        ),
        v2!(room_halfsize.x - wall_thickness, wall_thickness),
        cfg,
    );
    // left
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(
            center
                + v2!(
                    -room_halfsize.x - wall_thickness,
                    -room_halfsize.y - wall_thickness
                ),
        ),
        v2!(wall_thickness, room_halfsize.y + wall_thickness),
        cfg,
    );
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(
            center + v2!(-room_halfsize.x - wall_thickness, 2.0 * wall_thickness),
        ),
        v2!(wall_thickness, room_halfsize.y - wall_thickness),
        cfg,
    );
    // right
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(center + v2!(room_halfsize.x, -room_halfsize.y)),
        v2!(wall_thickness, room_halfsize.y),
        cfg,
    );
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(center + v2!(room_halfsize.x, 2.0 * wall_thickness)),
        v2!(wall_thickness, room_halfsize.y - wall_thickness),
        cfg,
    );

    // Central
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(center + v2!(-room_halfsize.x * 0.5, room_halfsize.y * 0.2)),
        v2!(wall_thickness * 4.0, wall_thickness),
        cfg,
    );
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(center + v2!(room_halfsize.x * 0.2, -room_halfsize.y * 0.2)),
        v2!(wall_thickness * 4.0, wall_thickness),
        cfg,
    );
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(center + v2!(-room_halfsize.x * 0.3, -room_halfsize.y * 0.65)),
        v2!(wall_thickness * 4.0, wall_thickness),
        cfg,
    );

    create_room_lights(
        center,
        room_halfsize,
        wall_thickness,
        &mut level.lights,
        cfg,
    );
}

fn create_room_lights(
    room_center: Vec2f,
    room_halfsize: Vec2f,
    wall_thickness: f32,
    lights: &mut Lights,
    cfg: &inle_cfg::Config,
) {
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
        radius: 550.,
        attenuation: 0.5,
        color: colors::DARK_ORANGE,
        intensity: 0.5,
    }));

    // top
    lights.queue_command(Light_Command::Add_Rect_Light(Rect_Light {
        rect: Rect::new(
            -room_halfsize.x,
            -room_halfsize.y + wall_thickness,
            2.0 * room_halfsize.x,
            1.,
        ) + room_center,
        radius: 50.,
        attenuation: 1.0,
        color: colors::DARK_ORANGE,
        intensity: 0.5,
    }));

    // bot
    lights.queue_command(Light_Command::Add_Rect_Light(Rect_Light {
        rect: Rect::new(
            -room_halfsize.x,
            room_halfsize.y - wall_thickness,
            2.0 * room_halfsize.x,
            1.,
        ) + room_center,
        radius: 50.,
        attenuation: 1.0,
        color: colors::DARK_ORANGE,
        intensity: 0.5,
    }));

    // left
    lights.queue_command(Light_Command::Add_Rect_Light(Rect_Light {
        rect: Rect::new(
            -room_halfsize.x + wall_thickness,
            -room_halfsize.y,
            1.,
            2.0 * room_halfsize.y,
        ) + room_center,
        radius: 50.,
        attenuation: 1.0,
        color: colors::DARK_ORANGE,
        intensity: 0.5,
    }));

    // right
    lights.queue_command(Light_Command::Add_Rect_Light(Rect_Light {
        rect: Rect::new(
            room_halfsize.x - wall_thickness,
            -room_halfsize.y,
            1.,
            2.0 * room_halfsize.y,
        ) + room_center,
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
