// @Temporary
#![allow(warnings)]

use crate::collisions::Game_Collision_Layer;
use crate::gfx::multi_sprite_animation_system::{Animation_Track, C_Multi_Renderable_Animation};
use crate::gfx::shaders::*;
use crate::systems::controllable_system::C_Controllable;
use crate::systems::gravity_system::C_Gravity;
use crate::systems::ground_detection_system::C_Ground_Detection;
//use crate::systems::pixel_collision_system::C_Texture_Collider;
use inle_cfg::{Cfg_Var, Config};
use inle_core::env::Env_Info;
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::{Ecs_World, Entity};
use inle_gfx::components::{C_Animated_Sprite, C_Multi_Renderable, C_Renderable};
use inle_gfx::material::Material;
use inle_gfx::render;
use inle_math::rect::Rect;
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use inle_physics::collider::{C_Collider, Collider, Collision_Shape};
use inle_physics::phys_world::{Phys_Data, Physics_World};
use inle_resources::gfx::{shader_path, tex_path, Gfx_Resources, Shader_Cache};

#[cfg(debug_assertions)]
use {
    crate::debug::entity_debug::C_Debug_Data,
    crate::debug::systems::position_history_system::C_Position_History, std::collections::HashMap,
    std::sync::Mutex,
};

#[cfg(debug_assertions)]
fn next_name(name: &'static str) -> String {
    lazy_static! {
        static ref N: Mutex<HashMap<&'static str, usize>> = Mutex::new(HashMap::new());
    }

    let mut guard = N.lock().unwrap();
    let n_ref = guard.entry(name).or_insert(0);
    let n = *n_ref;
    *n_ref += 1;
    format!("{}_{}", name, n)
}

#[cfg(debug_assertions)]
fn add_debug_data<'a>(world: &'a mut Ecs_World, entity: Entity, name: &'static str) {
    if world.has_component::<C_Spatial2D>(entity) {
        world.add_component(
            entity,
            C_Position_History::new(std::time::Duration::from_micros(200), 0.1),
        );
    }
    world.add_component(
        entity,
        C_Debug_Data {
            entity_name: next_name(name).as_str().into(),
            ..Default::default()
        },
    );
}

pub fn create_jelly(
    world: &mut Ecs_World,
    phys_world: &mut Physics_World,
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    cfg: &Config,
    transform: &Transform2D,
    make_controllable: bool,
) -> Entity {
    const N_ANIM_FRAMES: i32 = 3;

    let entity = world.new_entity();
    let renderable = C_Renderable::new_with_diffuse(gres, env, "jelly.png")
        .with_normals(gres, env, "jelly_n.png")
        .with_cast_shadows(true)
        .with_shininess(10.0)
        .with_shader(shader_cache, env, SHD_SPRITE_WITH_NORMALS)
        .with_n_frames(N_ANIM_FRAMES);
    world.add_component(entity, renderable);

    let tex = renderable.material.texture;
    let tex = gres.get_texture_mut(tex);
    render::set_texture_smooth(tex, false);

    // @Temporary
    if make_controllable {
        world.add_component(
            entity,
            C_Controllable {
                acceleration: Cfg_Var::new("game/gameplay/player/acceleration", cfg),
                jump_impulse: Cfg_Var::new("game/gameplay/player/jump_impulse", cfg),
                dampening: Cfg_Var::new("game/gameplay/player/dampening", cfg),
                horiz_max_speed: Cfg_Var::new("game/gameplay/player/horiz_max_speed", cfg),
                vert_max_speed: Cfg_Var::new("game/gameplay/player/vert_max_speed", cfg),
                max_jumps: Cfg_Var::new("game/gameplay/player/max_jumps", cfg),
                ..Default::default()
            },
        );
    }

    world.add_component(
        entity,
        C_Spatial2D {
            transform: *transform,
            ..Default::default()
        },
    );

    world.add_component(
        entity,
        C_Gravity {
            acceleration: Cfg_Var::new("game/gameplay/player/gravity", cfg),
        },
    );

    let (sw, sh) = render::get_texture_size(gres.get_texture(renderable.material.texture));
    let cld = Collider {
        shape: {
            let width = transform.scale().x * (sw / N_ANIM_FRAMES as u32) as f32;
            let height = transform.scale().y * sh as f32;
            Collision_Shape::Rect { width, height }
        },
        layer: Game_Collision_Layer::Entities as _,
        entity,
        ..Default::default()
    };
    let phys_data = Phys_Data {
        inv_mass: 1.,
        restitution: 0.9,
        static_friction: 0.5,
        dyn_friction: 0.3,
    };
    let phys_body = phys_world.new_physics_body_with_rigidbody(cld, entity, phys_data);

    world.add_component(
        entity,
        C_Collider {
            phys_body_handle: phys_body,
        },
    );
    world.add_component(entity, C_Ground_Detection::default());

    world.add_component(
        entity,
        C_Animated_Sprite {
            n_frames: N_ANIM_FRAMES as _,
            frame_time: 0.12,
            ..Default::default()
        },
    );

    #[cfg(debug_assertions)]
    {
        add_debug_data(world, entity, "Jelly");
    }

    entity
}

pub fn create_drill(
    world: &mut Ecs_World,
    phys_world: &mut Physics_World,
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    _cfg: &Config,
    transform: &Transform2D,
) -> Entity {
    let entity = world.new_entity();
    let shader = shader_cache.load_shader(&shader_path(env, SHD_SPRITE_WITH_NORMALS));

    let mut multi_rend = C_Multi_Renderable::default();

    {
        let texture = gres.load_texture(&tex_path(env, "drill_bottom.png"));
        let normals = gres.load_texture(&tex_path(env, "drill_bottom_n.png"));
        let (sw, sh) = render::get_texture_size(gres.get_texture(texture));
        multi_rend.add(C_Renderable {
            material: Material {
                texture,
                normals,
                shader,
                shininess: Material::encode_shininess(200.0),
                cast_shadows: true,
                ..Default::default()
            },
            rect: Rect::new(0, 0, sw as i32, sh as i32),
            ..Default::default()
        });
    }
    let (sw, sh) = {
        let texture = gres.load_texture(&tex_path(env, "drill_center.png"));
        let normals = gres.load_texture(&tex_path(env, "drill_center_n.png"));
        let (sw, sh) = render::get_texture_size(gres.get_texture(texture));
        multi_rend.add(C_Renderable {
            material: Material {
                texture,
                normals,
                shader,
                shininess: Material::encode_shininess(200.0),
                cast_shadows: true,
                ..Default::default()
            },
            rect: Rect::new(0, 0, sw as i32, sh as i32),
            ..Default::default()
        });
        (sw, sh)
    };
    {
        let texture = gres.load_texture(&tex_path(env, "drill_top.png"));
        let normals = gres.load_texture(&tex_path(env, "drill_top_n.png"));
        let (sw, sh) = render::get_texture_size(gres.get_texture(texture));
        multi_rend.add(C_Renderable {
            material: Material {
                texture,
                normals,
                shader,
                shininess: Material::encode_shininess(200.0),
                cast_shadows: true,
                ..Default::default()
            },
            rect: Rect::new(0, 0, sw as i32, sh as i32),
            z_index: 1,
            ..Default::default()
        });
    }

    world.add_component(entity, multi_rend);

    let mut mr_anim = C_Multi_Renderable_Animation::default();
    // Bottom
    mr_anim.anim_tracks_x[0] = Animation_Track::Sinusoidal {
        freq_hz: 100.,
        amplitude: 5.,
        phase: 0.,
        exp: 1,
    };
    // Center
    mr_anim.anim_tracks_x[1] = Animation_Track::Sinusoidal {
        freq_hz: 30.,
        amplitude: 2.,
        phase: 0.3,
        exp: 2,
    };
    // Top
    mr_anim.anim_tracks_x[2] = Animation_Track::Sinusoidal {
        freq_hz: 40.,
        amplitude: 1.,
        phase: 0.7,
        exp: 3,
    };
    world.add_component(entity, mr_anim);

    world.add_component(
        entity,
        C_Spatial2D {
            transform: *transform,
            ..Default::default()
        },
    );

    let cld = Collider {
        shape: {
            let width = sw as f32 * transform.scale().x;
            let height = sh as f32 * transform.scale().y;
            Collision_Shape::Rect { width, height }
        },
        layer: Game_Collision_Layer::Entities as _,
        ..Default::default()
    };
    world.add_component(
        entity,
        C_Collider {
            phys_body_handle: phys_world.new_physics_body_with_rigidbody(
                cld,
                entity,
                Phys_Data::default(),
            ),
        },
    );

    #[cfg(debug_assertions)]
    {
        add_debug_data(world, entity, "Drill");
    }

    entity
}

pub fn create_sky(
    world: &mut Ecs_World,
    phys_world: &mut Physics_World,
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    _cfg: &Config,
) {
    let sky = world.new_entity();

    {
        let mut spatial = C_Spatial2D::default();
        spatial.transform.set_position(0., -370.);
        world.add_component(sky, spatial);
    }

    let renderable = C_Renderable::new_with_diffuse(gres, env, "sky.png").with_shader(
        shader_cache,
        env,
        SHD_SPRITE_UNLIT,
    );
    let texture = renderable.material.texture;
    world.add_component(sky, renderable);

    let (sw, sh) = render::get_texture_size(gres.get_texture(texture));

    let cld = Collider {
        shape: Collision_Shape::Rect {
            width: sw as f32,
            height: sh as f32,
        },
        layer: Game_Collision_Layer::Sky as _,
        ..Default::default()
    };

    world.add_component(
        sky,
        C_Collider {
            phys_body_handle: phys_world.new_physics_body_with_rigidbody(
                cld,
                sky,
                Phys_Data {
                    inv_mass: 0.,
                    ..Default::default()
                },
            ),
        },
    );

    #[cfg(debug_assertions)]
    {
        add_debug_data(world, sky, "Sky");
    }
}

pub fn create_terrain(
    world: &mut Ecs_World,
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    _cfg: &Config,
) {
    let gnd = world.new_entity();

    {
        let mut t = C_Spatial2D::default();
        t.transform.set_position(0., 600.);
        world.add_component(gnd, t);
    }

    //let rend = world.add_component(
    //gnd,
    //C_Renderable::new_with_diffuse(gres, env, "ground3.png")
    //.with_shader(shader_cache, env, SHD_SPRITE_FLAT)
    ////.with_shader(shader_cache, env, SHD_TERRAIN)
    //.with_shininess(0.2)
    //.with_z_index(1),
    //);

    //let texture = rend.material.texture;
    //world.add_component(
    //gnd,
    //C_Texture_Collider {
    //texture,
    //layer: Game_Collision_Layer::Ground as _,
    //},
    //);

    #[cfg(debug_assertions)]
    {
        add_debug_data(world, gnd, "Terrain");
    }
}

pub fn create_background(
    world: &mut Ecs_World,
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    _cfg: &Config,
) {
    let ground = world.new_entity();
    let mut rend = C_Renderable::new_with_diffuse(gres, env, "ground.png")
        .with_shader(shader_cache, env, SHD_SPRITE_FLAT)
        .with_z_index(-1);
    let texture = gres.get_texture_mut(rend.material.texture);
    let (sw, sh) = render::get_texture_size(texture);
    rend.rect = Rect::new(0, 0, sw as i32 * 100, sh as i32 * 100);
    render::set_texture_repeated(texture, true);

    world.add_component(ground, rend);
    world.add_component(ground, C_Spatial2D::default());

    #[cfg(debug_assertions)]
    {
        add_debug_data(world, ground, "Background");
    }
}

pub fn create_room(
    world: &mut Ecs_World,
    phys_world: &mut Physics_World,
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    cfg: &Config,
) {
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(v2!(0.0, 400.)),
        v2!(1600., 400.),
        cfg,
    );
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(v2!(0.0, -400.)),
        v2!(1600., 400.),
        cfg,
    );
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(v2!(-550.0, 0.0)),
        v2!(500., 400.),
        cfg,
    );
    create_wall(
        world,
        phys_world,
        gres,
        shader_cache,
        env,
        &Transform2D::from_pos(v2!(550.0, 0.0)),
        v2!(500., 400.),
        cfg,
    );
}

pub fn create_wall(
    world: &mut Ecs_World,
    phys_world: &mut Physics_World,
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    transform: &Transform2D,
    wall_size: Vec2f,
    _cfg: &Config,
) {
    let wall = world.new_entity();

    let mut renderable = C_Renderable::new_with_diffuse(gres, env, "wall.png")
        .with_normals(gres, env, "wall_n.png")
        .with_shader(shader_cache, env, SHD_SPRITE_WITH_NORMALS)
        // NOTE: the wall's pivot, for convenience, is top-left.
        .with_local_transform(&Transform2D::from_pos(wall_size * 0.5));
    renderable.rect.width = wall_size.x as i32;
    renderable.rect.height = wall_size.y as i32;
    let texture = renderable.material.texture;
    let normals = renderable.material.normals;
    world.add_component(wall, renderable);

    {
        let tex = gres.get_texture_mut(texture);
        render::set_texture_repeated(tex, true);
    }
    {
        let tex = gres.get_texture_mut(normals);
        render::set_texture_repeated(tex, true);
    }

    let cld = Collider {
        shape: Collision_Shape::Rect {
            width: transform.scale().x * wall_size.x,
            height: transform.scale().y * wall_size.y,
        },
        layer: Game_Collision_Layer::Ground as _,
        // NOTE: the wall's pivot, for convenience, is top-left.
        offset: wall_size * 0.5,
        ..Default::default()
    };

    world.add_component(
        wall,
        C_Collider {
            phys_body_handle: phys_world.new_physics_body_with_rigidbody(
                cld,
                wall,
                Phys_Data {
                    inv_mass: 0.,
                    ..Default::default()
                },
            ),
        },
    );

    #[cfg(debug_assertions)]
    {
        add_debug_data(world, wall, "Wall");
    }

    {
        let mut spatial = C_Spatial2D::default();
        spatial.transform = *transform;
        world.add_component(wall, spatial);
    }
}
