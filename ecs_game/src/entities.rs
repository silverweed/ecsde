use crate::collisions::Game_Collision_Layer;
use crate::debug::entity_debug::C_Debug_Data;
use crate::gfx::multi_sprite_animation_system::{Animation_Track, C_Multi_Renderable_Animation};
use crate::gfx::shaders::*;
use crate::systems::controllable_system::C_Controllable;
use crate::systems::ground_collision_calculation_system::C_Ground;
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

pub fn create_jelly(
    world: &mut Ecs_World,
    gres: &mut Gfx_Resources,
    shader_cache: &mut Shader_Cache,
    env: &Env_Info,
    cfg: &Config,
    transform: &Transform2D,
    make_controllable: bool,
) -> Entity {
    const N_ANIM_FRAMES: usize = 3;

    let shd_sprite_with_normals =
        shader_cache.load_shader(&shader_path(env, SHD_SPRITE_WITH_NORMALS));
    let texture = gres.load_texture(&tex_path(&env, "jelly.png"));
    let normals = gres.load_texture(&tex_path(&env, "jelly_n.png"));
    let (sw, sh) = render::get_texture_size(gres.get_texture(texture));

    let entity = world.new_entity();
    {
        world.add_component(
            entity,
            C_Renderable {
                material: Material {
                    texture,
                    normals,
                    shader: shd_sprite_with_normals,
                    shininess: Material::encode_shininess(10.0),
                    cast_shadows: true,
                    ..Default::default()
                },
                rect: Rect::new(0, 0, sw as i32 / (N_ANIM_FRAMES as i32), sh as i32),
                ..Default::default()
            },
        );
    }

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
        world.add_component(entity, C_Debug_Data::default());
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

    let texture = gres.load_texture(&tex_path(&env, "rock.png"));
    let (sw, sh) = render::get_texture_size(gres.get_texture(texture));

    world.add_component(
        rock,
        C_Renderable {
            material: Material {
                texture,
                ..Default::default()
            },
            rect: Rect::new(0, 0, sw as _, sh as _),
            z_index: 1,
            ..Default::default()
        },
    );

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
        world.add_component(entity, C_Debug_Data::default());
    }

    entity
}
