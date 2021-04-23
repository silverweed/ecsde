use crate::light::{Lights, Point_Light, Rect_Light};
use crate::material::Material;
use crate::render::{self, Primitive_Type};
use crate::vbuf_holder::Vertex_Buffer_Holder;
use inle_alloc::temp;
use inle_common::colors::{self, Color, Color3};
use inle_gfx_backend::render::{Shader, Texture, Vertex};
use inle_gfx_backend::render_window::Render_Window_Handle;
use inle_math::angle::Angle;
use inle_math::math::{lerp, lerp_clamped};
use inle_math::matrix::Matrix3;
use inle_math::rect::Rect;
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use inle_resources::gfx::{Gfx_Resources, Shader_Cache};
use rayon::prelude::*;
use smallvec::SmallVec;
use std::cmp;
use std::collections::{BTreeMap, HashMap};

const SHADOWS_PER_ENTITY: usize = 4;
const VERTICES_PER_SPRITE: usize = 6;

struct Sprite_Batch {
    pub vbuffer: Vertex_Buffer_Holder,
    pub shadow_vbuffer: Option<Vertex_Buffer_Holder>,
    pub sprites: Vec<Sprite>,
}

#[derive(Default)]
pub struct Batches {
    textures_ws: BTreeMap<super::Z_Index, HashMap<Material, Sprite_Batch>>,
}

fn null_vertex() -> Vertex {
    render::new_vertex(v2!(0., 0.), colors::TRANSPARENT, v2!(0., 0.))
}

#[cfg(debug_assertions)]
fn invalid_vertex() -> Vertex {
    render::new_vertex(
        v2!(-12_345.67, 9_876.543),
        colors::rgba(42, 42, 42, 42),
        v2!(42., 42.),
    )
}

struct Sprite {
    pub tex_rect: Rect<i32>,
    pub color: Color,
    pub transform: Transform2D,
}

pub(super) fn add_texture_ws(
    window: &mut Render_Window_Handle,
    batches: &mut Batches,
    material: Material,
    tex_rect: &Rect<i32>,
    color: Color,
    transform: &Transform2D,
    z_index: super::Z_Index,
) {
    let z_index_texmap = {
        trace!("get_z_texmap");
        batches
            .textures_ws
            .entry(z_index)
            .or_insert_with(HashMap::new)
    };

    let sprite_batches = {
        trace!("get_sprite_batches");
        &mut z_index_texmap
            .entry(material)
            .or_insert_with(|| {
                ldebug!("creating buffer for material {:?}", material);
                Sprite_Batch {
                    vbuffer: Vertex_Buffer_Holder::with_initial_vertex_count(
                        window,
                        6 * 48,
                        Primitive_Type::Triangles,
                        #[cfg(debug_assertions)]
                        format!("{:?}", material),
                    ),
                    shadow_vbuffer: if material.cast_shadows {
                        Some(Vertex_Buffer_Holder::with_initial_vertex_count(
                            window,
                            6 * 4 * 48,
                            Primitive_Type::Triangles,
                            #[cfg(debug_assertions)]
                            format!("{:?}", material),
                        ))
                    } else {
                        None
                    },
                    sprites: vec![],
                }
            })
            .sprites
    };

    {
        trace!("push_tex");
        sprite_batches.push(Sprite {
            tex_rect: *tex_rect,
            color,
            transform: *transform,
        });
    }
}

pub fn clear_batches(batches: &mut Batches) {
    trace!("clear_batches");
    batches
        .textures_ws
        .values_mut()
        .for_each(|m| m.values_mut().for_each(|batch| batch.sprites.clear()));
}

#[inline(always)]
/// This returns the vec4 value that will be put into the vertex color.
/// It contains:
///    r: rotation high byte
///    g: rotation low byte
///    b: empty
///    a: vertex alpha
fn encode_rot_and_alpha_as_color(rot: Angle, alpha: u8) -> Color {
    const TAU: f32 = std::f32::consts::PI * 2.0;
    const MAX_ENCODED: u32 = u16::MAX as u32;

    let rad = rot.as_rad_0tau();
    let encoded_rad = (rad * MAX_ENCODED as f32 / TAU) as u32;
    Color {
        r: ((encoded_rad >> 8) & 0xFF) as u8,
        g: (encoded_rad & 0xFF) as u8,
        b: 0,
        a: alpha,
    }
}

macro_rules! set_point_light_uniforms {
    ($idx: expr, $shader: expr, $pl: expr) => {
        set_uniform(
            $shader,
            c_str!(concat!("point_lights[", $idx, "].position")),
            $pl.position,
        );
        set_uniform(
            $shader,
            c_str!(concat!("point_lights[", $idx, "].color")),
            Color3::from($pl.color),
        );
        set_uniform(
            $shader,
            c_str!(concat!("point_lights[", $idx, "].radius")),
            $pl.radius,
        );
        set_uniform(
            $shader,
            c_str!(concat!("point_lights[", $idx, "].attenuation")),
            $pl.attenuation,
        );
        set_uniform(
            $shader,
            c_str!(concat!("point_lights[", $idx, "].intensity")),
            $pl.intensity,
        );
    };
}

macro_rules! set_rect_light_uniforms {
    ($idx: expr, $shader: expr, $rl: expr) => {
        set_uniform(
            $shader,
            c_str!(concat!("rect_lights[", $idx, "].pos_min")),
            $rl.rect.pos_min(),
        );
        set_uniform(
            $shader,
            c_str!(concat!("rect_lights[", $idx, "].pos_max")),
            $rl.rect.pos_max(),
        );
        set_uniform(
            $shader,
            c_str!(concat!("rect_lights[", $idx, "].color")),
            Color3::from($rl.color),
        );
        set_uniform(
            $shader,
            c_str!(concat!("rect_lights[", $idx, "].radius")),
            $rl.radius,
        );
        set_uniform(
            $shader,
            c_str!(concat!("rect_lights[", $idx, "].attenuation")),
            $rl.attenuation,
        );
        set_uniform(
            $shader,
            c_str!(concat!("rect_lights[", $idx, "].intensity")),
            $rl.intensity,
        );
    };
}

fn set_shader_uniforms(
    shader: &mut Shader,
    material: &Material,
    gres: &Gfx_Resources,
    lights: &Lights,
    texture: &Texture,
    view_projection: &Matrix3<f32>,
) {
    use super::set_uniform;

    set_uniform(shader, c_str!("tex"), texture);
    set_uniform(shader, c_str!("vp"), view_projection);

    if material.normals.is_some() {
        let normals = gres.get_texture(material.normals);
        set_uniform(shader, c_str!("normals"), normals);
    }
    set_uniform(
        shader,
        c_str!("ambient_light.color"),
        Color3::from(lights.ambient_light.color),
    );
    set_uniform(
        shader,
        c_str!("ambient_light.intensity"),
        lights.ambient_light.intensity,
    );
    // @Cleanup: currently unused: was used by old terrain shader
    //let (tex_w, tex_h) = super::get_texture_size(texture);
    //set_uniform(
    //shader,
    //c_str!("texture_size"),
    //v2!(tex_w as f32, tex_h as f32),
    //);

    {
        let pls = &lights.point_lights;
        let pld = Point_Light::default();
        set_point_light_uniforms!(0, shader, pls.get(0).unwrap_or(&pld));
        set_point_light_uniforms!(1, shader, pls.get(1).unwrap_or(&pld));
        set_point_light_uniforms!(2, shader, pls.get(2).unwrap_or(&pld));
        set_point_light_uniforms!(3, shader, pls.get(3).unwrap_or(&pld));
    }

    {
        let rls = &lights.rect_lights;
        let rld = Rect_Light::default();
        set_rect_light_uniforms!(0, shader, rls.get(0).unwrap_or(&rld));
        set_rect_light_uniforms!(1, shader, rls.get(1).unwrap_or(&rld));
        set_rect_light_uniforms!(2, shader, rls.get(2).unwrap_or(&rld));
        set_rect_light_uniforms!(3, shader, rls.get(3).unwrap_or(&rld));
    }

    set_uniform(
        shader,
        c_str!("shininess"),
        Material::decode_shininess(material.shininess),
    );
    set_uniform(
        shader,
        c_str!("specular_color"),
        Color3::from(material.specular_color),
    );
}

#[derive(Copy, Clone)]
pub struct Batcher_Draw_Params {
    pub enable_shaders: bool,
    pub enable_shadows: bool,
}

impl Default for Batcher_Draw_Params {
    fn default() -> Self {
        Self {
            enable_shaders: true,
            enable_shadows: true,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_batches(
    window: &mut Render_Window_Handle,
    gres: &Gfx_Resources,
    batches: &mut Batches,
    shader_cache: &mut Shader_Cache,
    camera: &Transform2D,
    lights: &Lights,
    draw_params: Batcher_Draw_Params,
    frame_alloc: &mut temp::Temp_Allocator,
) {
    trace!("draw_all_batches");

    let n_threads = rayon::current_num_threads();
    let view_projection = get_vp_matrix(window, camera);

    // for each Z-index...
    for sprite_map in batches.textures_ws.values_mut() {
        // for each texture/shader...
        for (material, batch) in sprite_map {
            let vbuffer = &mut batch.vbuffer;
            let shadow_vbuffer = &mut batch.shadow_vbuffer;
            let sprites = &mut batch.sprites;

            let n_sprites = sprites.len();
            if n_sprites == 0 {
                // @Speed: right now we don't delete this batch from the tex_map, but it may be worth doing so.
                continue;
            }

            let texture = gres.get_texture(material.texture);

            let shader = if draw_params.enable_shaders {
                material.shader.map(|id| {
                    let shader = shader_cache.get_shader_mut(Some(id));
                    set_shader_uniforms(shader, material, gres, lights, texture, &view_projection);
                    shader
                })
            } else {
                None
            };
            let has_shader = shader.is_some();

            let cast_shadows = draw_params.enable_shadows && material.cast_shadows;
            let shadow_data =
                collect_entity_shadow_data(lights, sprites.iter(), cast_shadows, frame_alloc);

            let n_sprites_per_chunk = cmp::min(n_sprites, n_sprites / n_threads + 1);

            let n_vertices_without_shadows = (n_sprites * VERTICES_PER_SPRITE) as u32;
            debug_assert!(
                n_vertices_without_shadows as usize
                    + (shadow_data.len() * VERTICES_PER_SPRITE * SHADOWS_PER_ENTITY)
                    <= std::u32::MAX as usize
            );
            let n_shadow_vertices =
                (shadow_data.len() * VERTICES_PER_SPRITE * SHADOWS_PER_ENTITY) as u32;
            let n_vertices = n_vertices_without_shadows + n_shadow_vertices;

            // This buffer holds both regular vertices and shadow vertices
            let mut vertices = temp::excl_temp_array(frame_alloc);
            unsafe {
                vertices.alloc_additional_uninit(n_vertices as usize);
            }
            let (vertices, shadow_vertices) =
                vertices.split_at_mut(n_vertices_without_shadows as usize);

            #[cfg(debug_assertions)]
            {
                for vert in vertices.iter_mut() {
                    *vert = invalid_vertex();
                }
            }

            let n_vert_per_chunk = n_sprites_per_chunk * VERTICES_PER_SPRITE;
            let vert_chunks = vertices.par_chunks_mut(n_vert_per_chunk);

            // Ensure the vbuffer has enough room to write in
            if n_vertices_without_shadows > super::vbuf_max_vertices(&vbuffer.vbuf) {
                vbuffer.grow(window, n_vertices_without_shadows);
            }
            debug_assert!(n_vertices_without_shadows <= super::vbuf_max_vertices(&vbuffer.vbuf));

            {
                trace!("sprite_batch_ws");

                if cast_shadows {
                    fill_vertices_with_shadows(
                        window,
                        texture,
                        sprites,
                        vert_chunks,
                        n_sprites_per_chunk,
                        has_shader,
                        shadow_vbuffer.as_mut().unwrap(),
                        shadow_vertices,
                        n_shadow_vertices,
                        &shadow_data,
                    );
                } else {
                    fill_vertices(
                        texture,
                        sprites,
                        vert_chunks,
                        n_sprites_per_chunk,
                        has_shader,
                    );
                }
            }

            #[cfg(debug_assertions)]
            {
                // Sanity check: verify we wrote all the vertices
                // @Robustness: should be asserting equality, but Vertex has no PartialEq impl.
                fn is_invalid_vertex(v: &Vertex) -> bool {
                    let nv = invalid_vertex();
                    v.position() == nv.position()
                        && v.color() == nv.color()
                        && v.tex_coords() == nv.tex_coords()
                }

                for (i, vert) in vertices.iter().enumerate() {
                    debug_assert!(!is_invalid_vertex(vert), "Vertex {} is invalid!", i);
                }

                if cast_shadows {
                    for (i, vert) in shadow_vertices.iter().enumerate() {
                        debug_assert!(!is_invalid_vertex(vert), "Shadow vertex {} is invalid!", i);
                    }
                }
            }

            if cast_shadows {
                trace!("batcher::cast_shadows");

                let shadow_vbuffer = shadow_vbuffer.as_mut().unwrap();
                shadow_vbuffer.update(shadow_vertices, n_shadow_vertices);

                render::render_vbuf_ws_with_texture(
                    window,
                    &shadow_vbuffer.vbuf,
                    &Transform2D::default(),
                    camera,
                    texture,
                );
            }

            vbuffer.update(vertices, n_vertices_without_shadows);

            if let Some(shader) = shader.map(|s| s as &_) {
                render::render_vbuf_with_shader(window, &vbuffer.vbuf, shader);
            } else {
                render::render_vbuf_ws_with_texture(
                    window,
                    &vbuffer.vbuf,
                    &Transform2D::default(),
                    camera,
                    texture,
                );
            }
        }
    }
}

struct Entity_Shadow_Data {
    pub nearby_point_lights: SmallVec<[Point_Light; SHADOWS_PER_ENTITY]>,
}

fn collect_entity_shadow_data<'a>(
    lights: &Lights,
    sprites: impl Iterator<Item = &'a Sprite>,
    cast_shadows: bool,
    frame_alloc: &mut temp::Temp_Allocator,
) -> temp::Read_Only_Temp_Array<Entity_Shadow_Data> {
    let mut shadow_data = temp::excl_temp_array(frame_alloc);
    if cast_shadows {
        for tex in sprites {
            let mut nearby_point_lights = SmallVec::<[Point_Light; SHADOWS_PER_ENTITY]>::new();

            // @Speed: should lights be spatially accelerated?
            lights.get_all_point_lights_sorted_by_distance_within(
                tex.transform.position(),
                10000.,
                &mut nearby_point_lights,
                SHADOWS_PER_ENTITY,
            );
            debug_assert!(nearby_point_lights.len() <= SHADOWS_PER_ENTITY);
            nearby_point_lights.resize(SHADOWS_PER_ENTITY, Point_Light::default());
            shadow_data.push(Entity_Shadow_Data {
                nearby_point_lights,
            });
        }
    }

    unsafe { shadow_data.into_read_only() }
}

fn fill_vertices_with_shadows(
    window: &mut Render_Window_Handle,
    texture: &Texture,
    sprites: &[Sprite],
    vert_chunks: rayon::slice::ChunksMut<'_, Vertex>,
    n_sprites_per_chunk: usize,
    has_shader: bool,
    shadow_vbuffer: &mut Vertex_Buffer_Holder,
    shadow_vertices: &mut [Vertex],
    n_shadow_vertices: u32,
    shadow_data: &temp::Read_Only_Temp_Array<Entity_Shadow_Data>,
) {
    let n_vert_per_chunk = n_sprites_per_chunk * VERTICES_PER_SPRITE;

    if n_shadow_vertices > super::vbuf_max_vertices(&shadow_vbuffer.vbuf) {
        shadow_vbuffer.grow(window, n_shadow_vertices);
    }
    debug_assert!(n_shadow_vertices <= super::vbuf_max_vertices(&shadow_vbuffer.vbuf));

    #[cfg(debug_assertions)]
    {
        for vert in shadow_vertices.iter_mut() {
            *vert = invalid_vertex();
        }
    }
    #[cfg(not(debug_assertions))]
    {
        for vert in shadow_vertices.iter_mut() {
            *vert = null_vertex();
        }
    }

    debug_assert_eq!(shadow_vertices.len(), n_shadow_vertices as usize);
    let shadows_per_chunk = n_vert_per_chunk * SHADOWS_PER_ENTITY;
    let shadow_chunks = shadow_vertices.par_chunks_mut(shadows_per_chunk);

    let sprite_chunks = sprites.par_chunks(n_sprites_per_chunk);
    debug_assert_eq!(sprite_chunks.len(), vert_chunks.len());
    debug_assert_eq!(sprite_chunks.len(), shadow_chunks.len());

    sprite_chunks.zip(vert_chunks).zip(shadow_chunks).for_each(
        |((sprite_chunk, vert_chunk), shadow_chunk)| {
            for (i, sprite) in sprite_chunk.iter().enumerate() {
                let Sprite {
                    tex_rect,
                    transform,
                    color,
                } = sprite;

                let tex_size = render::get_texture_size(texture);
                let (tw, th) = (tex_size.0 as f32, tex_size.1 as f32);
                let uv: Rect<f32> = Rect::new(
                    tex_rect.x as f32 / tw,
                    tex_rect.y as f32 / th,
                    tex_rect.width as f32 / tw,
                    tex_rect.height as f32 / th,
                );
                let sprite_size = Vec2f::new(tex_rect.width as _, tex_rect.height as _);
                let render_transform = *transform;

                let color = if has_shader {
                    encode_rot_and_alpha_as_color(transform.rotation(), color.a)
                } else {
                    *color
                };

                // Note: beware of the order of multiplications!
                // Scaling the local positions must be done BEFORE multiplying the matrix!
                let p1 = render_transform * (sprite_size * v2!(-0.5, -0.5));
                let p2 = render_transform * (sprite_size * v2!(0.5, -0.5));
                let p3 = render_transform * (sprite_size * v2!(0.5, 0.5));
                let p4 = render_transform * (sprite_size * v2!(-0.5, 0.5));

                let v1 = super::new_vertex(p1, color, v2!(uv.x, uv.y));
                let v2 = super::new_vertex(p2, color, v2!(uv.x + uv.width, uv.y));
                let v3 = super::new_vertex(p3, color, v2!(uv.x + uv.width, uv.y + uv.height));
                let v4 = super::new_vertex(p4, color, v2!(uv.x, uv.y + uv.height));

                vert_chunk[i * 6] = v1;
                vert_chunk[i * 6 + 1] = v2;
                vert_chunk[i * 6 + 2] = v3;
                vert_chunk[i * 6 + 3] = v3;
                vert_chunk[i * 6 + 4] = v4;
                vert_chunk[i * 6 + 5] = v1;

                // @Incomplete: the shadow looks weird: it should be flipped in certain situations
                // and probably have some bias to not make the entity look like "floating"
                for (light_idx, light) in shadow_data[i].nearby_point_lights.iter().enumerate() {
                    debug_assert!(light_idx < 4);
                    let light_pos = light.position;
                    let recp_radius2 = 1.0 / (light.radius * light.radius);
                    let mut v = [v1, v2, v3, v4];
                    let diff = [
                        light_pos - p1,
                        light_pos - p2,
                        light_pos - p3,
                        light_pos - p4,
                    ];
                    let dist2 = [
                        diff[0].magnitude2(),
                        diff[1].magnitude2(),
                        diff[2].magnitude2(),
                        diff[3].magnitude2(),
                    ];
                    let min_d_sqr = dist2
                        .iter()
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap();

                    const SHADOW_MAX_VALUE: f32 = 50.0;

                    for v_idx in 0..4 {
                        v[v_idx].set_position(
                            v[v_idx].position()
                                - lerp_clamped(0.0, 1.0, dist2[v_idx] / min_d_sqr - 1.0)
                                    * diff[v_idx],
                        );

                        let t = (1.0 - dist2[v_idx] * recp_radius2).max(0.0);
                        v[v_idx].set_color(colors::rgba(
                            0,
                            0,
                            0,
                            lerp(0.0, SHADOW_MAX_VALUE, t * t) as u8,
                        ));
                    }

                    shadow_chunk[6 * (SHADOWS_PER_ENTITY * i + light_idx)] = v[0];
                    shadow_chunk[6 * (SHADOWS_PER_ENTITY * i + light_idx) + 1] = v[1];
                    shadow_chunk[6 * (SHADOWS_PER_ENTITY * i + light_idx) + 2] = v[2];
                    shadow_chunk[6 * (SHADOWS_PER_ENTITY * i + light_idx) + 3] = v[2];
                    shadow_chunk[6 * (SHADOWS_PER_ENTITY * i + light_idx) + 4] = v[3];
                    shadow_chunk[6 * (SHADOWS_PER_ENTITY * i + light_idx) + 5] = v[0];
                }

                #[cfg(debug_assertions)]
                {
                    for light_idx in shadow_data[i].nearby_point_lights.len()..4 {
                        for v_idx in 0..4 {
                            shadow_chunk[4 * (SHADOWS_PER_ENTITY * v_idx + light_idx) + v_idx] =
                                null_vertex();
                        }
                    }
                }
            }
        },
    );
}

fn fill_vertices(
    texture: &Texture,
    sprites: &[Sprite],
    vert_chunks: rayon::slice::ChunksMut<'_, Vertex>,
    n_sprites_per_chunk: usize,
    has_shader: bool,
) {
    sprites
        .par_chunks(n_sprites_per_chunk)
        .zip(vert_chunks)
        .for_each(|(sprite_chunk, vert_chunk)| {
            for (i, sprite) in sprite_chunk.iter().enumerate() {
                let Sprite {
                    tex_rect,
                    transform,
                    color,
                } = sprite;

                let tex_size = render::get_texture_size(texture);
                let (tw, th) = (tex_size.0 as f32, tex_size.1 as f32);
                let uv: Rect<f32> = Rect::new(
                    tex_rect.x as f32 / tw,
                    tex_rect.y as f32 / th,
                    tex_rect.width as f32 / tw,
                    tex_rect.height as f32 / th,
                );
                let sprite_size = Vec2f::new(tex_rect.width as _, tex_rect.height as _);
                let render_transform = *transform;

                // Encode rotation in color
                let color = if has_shader {
                    encode_rot_and_alpha_as_color(transform.rotation(), color.a)
                } else {
                    *color
                };

                // Note: beware of the order of multiplications!
                // Scaling the local positions must be done BEFORE multiplying the matrix!
                let p1 = render_transform * (sprite_size * v2!(-0.5, -0.5));
                let p2 = render_transform * (sprite_size * v2!(0.5, -0.5));
                let p3 = render_transform * (sprite_size * v2!(0.5, 0.5));
                let p4 = render_transform * (sprite_size * v2!(-0.5, 0.5));

                let v1 = super::new_vertex(p1, color, v2!(uv.x, uv.y));
                let v2 = super::new_vertex(p2, color, v2!(uv.x + uv.width, uv.y));
                let v3 = super::new_vertex(p3, color, v2!(uv.x + uv.width, uv.y + uv.height));
                let v4 = super::new_vertex(p4, color, v2!(uv.x, uv.y + uv.height));

                vert_chunk[i * 6] = v1;
                vert_chunk[i * 6 + 1] = v2;
                vert_chunk[i * 6 + 2] = v3;
                vert_chunk[i * 6 + 3] = v3;
                vert_chunk[i * 6 + 4] = v4;
                vert_chunk[i * 6 + 5] = v1;
            }
        });
}

fn get_vp_matrix(window: &Render_Window_Handle, camera: &Transform2D) -> Matrix3<f32> {
    let (width, height) = inle_win::window::get_window_target_size(window);
    let view = camera.inverse();
    let projection = Matrix3::new(
        2. / width as f32,
        0.,
        0.,
        0.,
        -2. / height as f32,
        0.,
        0.,
        0.,
        1.,
    );
    projection * view.get_matrix()
}
