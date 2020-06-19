use crate::collisions::Game_Collision_Layer;
use crate::gfx::multi_sprite_animation_system::{Animation_Track, C_Multi_Renderable_Animation};
use crate::gfx::shaders::*;
use crate::systems::controllable_system::C_Controllable;
use crate::systems::ground_collision_calculation_system::C_Ground;
use crate::systems::pixel_collision_system::C_Texture_Collider;
use ecs_engine::cfg::{Cfg_Var, Config};
use ecs_engine::collisions::collider::{C_Phys_Data, Collider, Collision_Shape};
use ecs_engine::common::rect::Rect;
use ecs_engine::common::transform::Transform2D;
use ecs_engine::core::env::Env_Info;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::components::gfx::{
    C_Animated_Sprite, C_Multi_Renderable, C_Renderable, Material,
};
use ecs_engine::ecs::ecs_world::{Ecs_World, Entity};
use ecs_engine::gfx::render;
use ecs_engine::resources::gfx::{shader_path, tex_path, Gfx_Resources, Shader_Cache};

#[cfg(debug_assertions)]
use crate::debug::entity_debug::C_Debug_Data;

pub fn create_jelly(
    world: &mut Ecs_World,
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

    // @Temporary
    if make_controllable {
        world.add_component(
            entity,
            C_Controllable {
                speed: Cfg_Var::new("game/gameplay/player_speed", cfg),
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

    let (sw, sh) = render::get_texture_size(gres.get_texture(renderable.material.texture));
    world.add_component(
        entity,
        Collider {
            shape: {
                let width = (sw / N_ANIM_FRAMES as u32) as f32;
                let height = sh as f32;
                Collision_Shape::Rect { width, height }
            },
            layer: Game_Collision_Layer::Entities as _,
            ..Default::default()
        },
    );

    world.add_component(
        entity,
        C_Phys_Data {
            inv_mass: 1.,
            restitution: 0.9,
            static_friction: 0.5,
            dyn_friction: 0.3,
        },
    );

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
        let debug = world.add_component(entity, C_Debug_Data::default());
        debug.entity_name = "Jelly";
    }

    entity
}

pub fn create_rock(
    world: &mut Ecs_World,
    gres: &mut Gfx_Resources,
    env: &Env_Info,
    transform: &Transform2D,
) -> Entity {
    let rock = world.new_entity();

    let renderable = C_Renderable::new_with_diffuse(gres, env, "rock.png").with_z_index(1);
    world.add_component(rock, renderable);

    world.add_component(
        rock,
        C_Spatial2D {
            transform: *transform,
            ..Default::default()
        },
    );

    world.add_component(rock, C_Ground::default());

    world.add_component(
        rock,
        C_Phys_Data {
            inv_mass: 0., // infinite mass
            restitution: 1.0,
            static_friction: 0.5,
            dyn_friction: 0.3,
        },
    );

    #[cfg(debug_assertions)]
    {
        let debug = world.add_component(rock, C_Debug_Data::default());
        debug.entity_name = "Rock";
    }

    rock
}

pub fn create_drill(
    world: &mut Ecs_World,
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
        let texture = gres.load_texture(&tex_path(&env, "drill_bottom.png"));
        let normals = gres.load_texture(&tex_path(&env, "drill_bottom_n.png"));
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
        let texture = gres.load_texture(&tex_path(&env, "drill_center.png"));
        let normals = gres.load_texture(&tex_path(&env, "drill_center_n.png"));
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
        let texture = gres.load_texture(&tex_path(&env, "drill_top.png"));
        let normals = gres.load_texture(&tex_path(&env, "drill_top_n.png"));
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

    let mr_anim = world.add_component(entity, C_Multi_Renderable_Animation::default());
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

    world.add_component(
        entity,
        C_Spatial2D {
            transform: *transform,
            ..Default::default()
        },
    );

    world.add_component(
        entity,
        Collider {
            shape: {
                let width = sw as f32 * transform.scale().x;
                let height = sh as f32 * transform.scale().y;
                Collision_Shape::Rect { width, height }
            },
            layer: Game_Collision_Layer::Entities as _,
            ..Default::default()
        },
    );

    world.add_component(entity, C_Phys_Data::default());

    #[cfg(debug_assertions)]
    {
        let debug = world.add_component(entity, C_Debug_Data::default());
        debug.entity_name = "Drill";
    }

    entity
}

pub fn create_sky(
    world: &mut Ecs_World,
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    _cfg: &Config,
) {
    let sky = world.new_entity();

    {
        let spatial = world.add_component(sky, C_Spatial2D::default());
        spatial.transform.set_position(0., -370.);
    }

    let renderable = world.add_component(
        sky,
        C_Renderable::new_with_diffuse(gres, env, "sky.png").with_shader(
            shader_cache,
            env,
            SHD_SPRITE_UNLIT,
        ),
    );

    let (sw, sh) = render::get_texture_size(gres.get_texture(renderable.material.texture));

    world.add_component(
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

    world.add_component(
        sky,
        C_Phys_Data {
            inv_mass: 0.,
            ..Default::default()
        },
    );

    #[cfg(debug_assertions)]
    {
        let debug = world.add_component(sky, C_Debug_Data::default());
        debug.entity_name = "Sky";
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
        let t = world.add_component(gnd, C_Spatial2D::default());
        t.transform.set_position(0., 600.);
    }

    let rend = world.add_component(
        gnd,
        C_Renderable::new_with_diffuse(gres, env, "ground3.png")
            .with_shader(shader_cache, env, SHD_TERRAIN)
            .with_shininess(0.2)
            .with_z_index(1),
    );

    let texture = rend.material.texture;

    world.add_component(
        gnd,
        C_Texture_Collider {
            texture,
            layer: Game_Collision_Layer::Ground as _,
        },
    );

    #[cfg(debug_assertions)]
    {
        let debug = world.add_component(gnd, C_Debug_Data::default());
        debug.entity_name = "Terrain";
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
    let rend = world.add_component(
        ground,
        C_Renderable::new_with_diffuse(gres, env, "ground.png")
            .with_shader(shader_cache, env, SHD_SPRITE_FLAT)
            .with_z_index(-1),
    );

    let texture = gres.get_texture_mut(rend.material.texture);
    let (sw, sh) = render::get_texture_size(texture);
    rend.rect = Rect::new(0, 0, sw as i32 * 100, sh as i32 * 100);
    render::set_texture_repeated(texture, true);

    world.add_component(ground, C_Spatial2D::default());

    #[cfg(debug_assertions)]
    {
        let debug = world.add_component(ground, C_Debug_Data::default());
        debug.entity_name = "Background";
    }
}
