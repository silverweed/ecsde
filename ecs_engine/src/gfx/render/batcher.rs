use super::backend;
use crate::alloc::temp;
use crate::common::angle::Angle;
use crate::common::colors::{self, Color};
use crate::common::math::{lerp, lerp_clamped};
use crate::common::rect::Rect;
use crate::common::transform::Transform2D;
use crate::common::vector::Vec2f;
use crate::ecs::components::gfx::Material;
use crate::gfx::light::{Lights, Point_Light};
use crate::gfx::render::{self, Shader, Texture, Vertex, Vertex_Buffer_Quads};
use crate::gfx::render_window::Render_Window_Handle;
use crate::resources::gfx::{Gfx_Resources, Shader_Cache};
use rayon::prelude::*;
use std::cmp;
use std::collections::{BTreeMap, HashMap};

struct Sprite_Batch {
    pub vbuffer: Vertex_Buffer_Holder,
    pub shadow_vbuffer: Option<Vertex_Buffer_Holder>,
    pub sprites: Vec<Sprite>,
}

#[derive(Default)]
pub struct Batches {
    textures_ws: BTreeMap<render::Z_Index, HashMap<Material, Sprite_Batch>>,
}

struct Vertex_Buffer_Holder {
    pub vbuf: Vertex_Buffer_Quads,
    #[cfg(debug_assertions)]
    id: Material,
}

// @WaitForStable make this const
fn null_vertex() -> Vertex {
    backend::new_vertex(v2!(0., 0.), colors::TRANSPARENT, v2!(0., 0.))
}

// @WaitForStable make this const
#[cfg(debug_assertions)]
fn invalid_vertex() -> Vertex {
    backend::new_vertex(
        v2!(-12_345.67, 9_876.543),
        colors::rgba(42, 42, 42, 42),
        v2!(42., 42.),
    )
}

impl Vertex_Buffer_Holder {
    pub fn with_initial_vertex_count(
        initial_cap: u32,
        #[cfg(debug_assertions)] id: Material,
    ) -> Self {
        Self {
            vbuf: render::start_draw_quads(initial_cap / 4),
            #[cfg(debug_assertions)]
            id,
        }
    }

    pub fn update(&mut self, vertices: &mut [Vertex], n_vertices: u32) {
        trace!("vbuf_update");

        debug_assert!(vertices.len() <= std::u32::MAX as usize);

        debug_assert!(
            n_vertices <= render::vbuf_max_vertices(&self.vbuf),
            "Can't hold all the vertices! {} / {}",
            n_vertices,
            render::vbuf_max_vertices(&self.vbuf)
        );

        // Zero all the excess vertices
        for vertex in vertices
            .iter_mut()
            .take(render::vbuf_cur_vertices(&self.vbuf) as usize)
            .skip(n_vertices as usize)
        {
            *vertex = null_vertex();
        }

        backend::update_vbuf(&mut self.vbuf, vertices, 0);
        render::set_vbuf_cur_vertices(&mut self.vbuf, vertices.len() as u32);
    }

    pub fn grow(&mut self, vertices_to_hold_at_least: u32) {
        let new_cap = vertices_to_hold_at_least.next_power_of_two();
        ldebug!(
            "Growing Vertex_Buffer_Holder {:?} to hold {} vertices ({} requested).",
            self.id,
            new_cap,
            vertices_to_hold_at_least
        );

        let mut new_vbuf = render::start_draw_quads(new_cap / 4);
        let _res = render::swap_vbuf(&mut new_vbuf, &mut self.vbuf);
        #[cfg(debug_assertions)]
        {
            debug_assert!(_res, "Vertex Buffer copying failed ({:?})!", self.id);
        }
    }
}

struct Sprite {
    pub tex_rect: Rect<i32>,
    pub color: Color,
    pub transform: Transform2D,
}

pub(super) fn add_texture_ws(
    batches: &mut Batches,
    material: Material,
    tex_rect: &Rect<i32>,
    color: Color,
    transform: &Transform2D,
    z_index: render::Z_Index,
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
                ldebug!("creating buffer for material {:?}", material,);
                Sprite_Batch {
                    vbuffer: Vertex_Buffer_Holder::with_initial_vertex_count(
                        48,
                        #[cfg(debug_assertions)]
                        material,
                    ),
                    shadow_vbuffer: if material.cast_shadows {
                        Some(Vertex_Buffer_Holder::with_initial_vertex_count(
                            48 * 4,
                            #[cfg(debug_assertions)]
                            material,
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
// NOTE: we're only using 2 bytes out of the 4 we have: we may fit more data in the future! (maybe an indexed color?)
fn encode_rot_as_color(rot: Angle) -> Color {
    const MAX_ENCODED: u32 = std::u16::MAX as u32;
    let rad = rot.as_rad_0tau();
    let encoded = (rad * MAX_ENCODED as f32 / std::f32::consts::PI * 0.5) as u32;
    Color {
        r: 0,
        g: 0,
        b: ((encoded >> 8) & 0xFF) as u8,
        a: (encoded & 0xFF) as u8,
    }
}

fn set_shader_uniforms(
    shader: &mut Shader,
    material: &Material,
    gres: &Gfx_Resources,
    lights: &Lights,
    texture: &Texture,
) {
    use render::{set_uniform_color, set_uniform_float, set_uniform_texture, set_uniform_vec2};

    if material.normals.is_some() {
        let normals = gres.get_texture(material.normals);
        set_uniform_texture(shader, "normals", normals);
    }
    set_uniform_color(shader, "ambient_light.color", lights.ambient_light.color);
    set_uniform_float(
        shader,
        "ambient_light.intensity",
        lights.ambient_light.intensity,
    );
    set_uniform_texture(shader, "texture", texture);
    let (tex_w, tex_h) = render::get_texture_size(texture);
    set_uniform_vec2(shader, "texture_size", v2!(tex_w as f32, tex_h as f32));
    for (i, pl) in lights.point_lights.iter().enumerate() {
        set_uniform_vec2(
            shader,
            &format!("point_lights[{}].position", i),
            pl.position,
        );
        set_uniform_color(shader, &format!("point_lights[{}].color", i), pl.color);
        set_uniform_float(shader, &format!("point_lights[{}].radius", i), pl.radius);
        set_uniform_float(
            shader,
            &format!("point_lights[{}].attenuation", i),
            pl.attenuation,
        );
        set_uniform_float(
            shader,
            &format!("point_lights[{}].intensity", i),
            pl.intensity,
        );
    }
    set_uniform_float(
        shader,
        "shininess",
        Material::decode_shininess(material.shininess),
    );
    set_uniform_color(shader, "specular_color", material.specular_color);
}

#[allow(clippy::too_many_arguments)]
pub fn draw_batches(
    window: &mut Render_Window_Handle,
    gres: &Gfx_Resources,
    batches: &mut Batches,
    shader_cache: &mut Shader_Cache,
    camera: &Transform2D,
    lights: &Lights,
    enable_shaders: bool,
    frame_alloc: &mut temp::Temp_Allocator,
) {
    trace!("draw_all_batches");

    const SHADOWS_PER_ENTITY: usize = 4;

    let n_threads = rayon::current_num_threads();

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

            let shader = if enable_shaders {
                material.shader.map(|id| {
                    let shader = shader_cache.get_shader_mut(Some(id));
                    set_shader_uniforms(shader, material, gres, lights, texture);
                    shader
                })
            } else {
                None
            };
            let has_shader = shader.is_some();

            let cast_shadows = material.cast_shadows;
            // @Temporary
            // @Speed! We can't use the temp array as it's not Send! Maybe we should make it Send (at least a read-only version of it)?
            let mut shadows = vec![];
            if cast_shadows {
                for tex in sprites.iter() {
                    let mut nearby_point_lights = Vec::with_capacity(SHADOWS_PER_ENTITY); // @Speed!

                    // @Speed: should lights be spatially accelerated?
                    lights.get_all_point_lights_sorted_by_distance_within(
                        tex.transform.position(),
                        10000.,
                        &mut nearby_point_lights,
                        SHADOWS_PER_ENTITY,
                    );
                    nearby_point_lights.resize(4, Point_Light::default());
                    shadows.push(nearby_point_lights);
                }
            }

            let n_sprites_per_chunk = cmp::min(n_sprites, n_sprites / n_threads + 1);

            let n_vertices_without_shadows = (n_sprites * 4) as u32;
            debug_assert!(
                n_vertices_without_shadows as usize + (shadows.len() * 4 * SHADOWS_PER_ENTITY)
                    <= std::u32::MAX as usize
            );
            let n_shadow_vertices = (shadows.len() * 4 * SHADOWS_PER_ENTITY) as u32;
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

            let n_vert_per_chunk = n_sprites_per_chunk * 4;
            let vert_chunks = vertices.par_iter_mut().chunks(n_vert_per_chunk);

            // Ensure the vbuffer has enough room to write in
            if n_vertices_without_shadows > render::vbuf_max_vertices(&vbuffer.vbuf) {
                vbuffer.grow(n_vertices_without_shadows);
            }
            debug_assert!(n_vertices_without_shadows <= render::vbuf_max_vertices(&vbuffer.vbuf));

            {
                trace!("sprite_batch_ws");

                if cast_shadows {
                    let shadow_vbuffer = shadow_vbuffer.as_mut().unwrap();
                    if n_shadow_vertices > render::vbuf_max_vertices(&shadow_vbuffer.vbuf) {
                        shadow_vbuffer.grow(n_shadow_vertices);
                    }
                    debug_assert!(
                        n_shadow_vertices <= render::vbuf_max_vertices(&shadow_vbuffer.vbuf)
                    );

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
                    let shadow_chunks = shadow_vertices.par_iter_mut().chunks(shadows_per_chunk);

                    let sprite_chunks = sprites.par_iter().chunks(n_sprites_per_chunk);
                    debug_assert_eq!(sprite_chunks.len(), vert_chunks.len());
                    debug_assert_eq!(sprite_chunks.len(), shadow_chunks.len());

                    sprite_chunks.zip(vert_chunks).zip(shadow_chunks).for_each(
                        |((sprite_chunk, mut vert_chunk), mut shadow_chunk)| {
                            for (i, sprite) in sprite_chunk.iter().enumerate() {
                                let Sprite {
                                    tex_rect,
                                    transform,
                                    color,
                                } = sprite;

                                let uv: Rect<f32> = (*tex_rect).into();
                                let sprite_size =
                                    Vec2f::new(tex_rect.width as _, tex_rect.height as _);
                                let render_transform = *transform;

                                // Encode rotation in color
                                let color = if has_shader {
                                    encode_rot_as_color(transform.rotation())
                                } else {
                                    *color
                                };

                                // Note: beware of the order of multiplications!
                                // Scaling the local positions must be done BEFORE multiplying the matrix!
                                let p1 = render_transform * (sprite_size * v2!(-0.5, -0.5));
                                let p2 = render_transform * (sprite_size * v2!(0.5, -0.5));
                                let p3 = render_transform * (sprite_size * v2!(0.5, 0.5));
                                let p4 = render_transform * (sprite_size * v2!(-0.5, 0.5));

                                let v1 = render::new_vertex(p1, color, v2!(uv.x, uv.y));
                                let v2 = render::new_vertex(p2, color, v2!(uv.x + uv.width, uv.y));
                                let v3 = render::new_vertex(
                                    p3,
                                    color,
                                    v2!(uv.x + uv.width, uv.y + uv.height),
                                );
                                let v4 = render::new_vertex(p4, color, v2!(uv.x, uv.y + uv.height));

                                *vert_chunk[i * 4] = v1;
                                *vert_chunk[i * 4 + 1] = v2;
                                *vert_chunk[i * 4 + 2] = v3;
                                *vert_chunk[i * 4 + 3] = v4;

                                // @Incomplete: the shadow looks weird: it should be flipped in certain situations
                                // and probably have some bias to not make the entity look like "floating"
                                for (light_idx, light) in shadows[i].iter().enumerate() {
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
                                        v[v_idx].position -= (lerp_clamped(
                                            0.0,
                                            1.0,
                                            dist2[v_idx] / min_d_sqr - 1.0,
                                        ) * diff[v_idx])
                                            .into();
                                        let t = (1.0 - dist2[v_idx] * recp_radius2).max(0.0);
                                        v[v_idx].color = colors::rgba(
                                            0,
                                            0,
                                            0,
                                            lerp(0.0, SHADOW_MAX_VALUE, t * t) as u8,
                                        )
                                        .into();

                                        *shadow_chunk
                                            [4 * (SHADOWS_PER_ENTITY * i + light_idx) + v_idx] =
                                            v[v_idx];
                                    }
                                }

                                #[cfg(debug_assertions)]
                                {
                                    for light_idx in shadows[i].len()..4 {
                                        for v_idx in 0..4 {
                                            *shadow_chunk[4
                                                * (SHADOWS_PER_ENTITY * v_idx + light_idx)
                                                + v_idx] = null_vertex();
                                        }
                                    }
                                }
                            }
                        },
                    );
                } else {
                    sprites
                        .par_iter()
                        .chunks(n_sprites_per_chunk)
                        .zip(vert_chunks)
                        .for_each(|(sprite_chunk, mut vert_chunk)| {
                            for (i, sprite) in sprite_chunk.iter().enumerate() {
                                let Sprite {
                                    tex_rect,
                                    transform,
                                    color,
                                } = sprite;

                                let uv: Rect<f32> = (*tex_rect).into();
                                let sprite_size =
                                    Vec2f::new(tex_rect.width as _, tex_rect.height as _);
                                let render_transform = *transform;

                                // Encode rotation in color
                                let color = if has_shader {
                                    encode_rot_as_color(transform.rotation())
                                } else {
                                    *color
                                };

                                // Note: beware of the order of multiplications!
                                // Scaling the local positions must be done BEFORE multiplying the matrix!
                                let p1 = render_transform * (sprite_size * v2!(-0.5, -0.5));
                                let p2 = render_transform * (sprite_size * v2!(0.5, -0.5));
                                let p3 = render_transform * (sprite_size * v2!(0.5, 0.5));
                                let p4 = render_transform * (sprite_size * v2!(-0.5, 0.5));

                                let v1 = render::new_vertex(p1, color, v2!(uv.x, uv.y));
                                let v2 = render::new_vertex(p2, color, v2!(uv.x + uv.width, uv.y));
                                let v3 = render::new_vertex(
                                    p3,
                                    color,
                                    v2!(uv.x + uv.width, uv.y + uv.height),
                                );
                                let v4 = render::new_vertex(p4, color, v2!(uv.x, uv.y + uv.height));

                                *vert_chunk[i * 4] = v1;
                                *vert_chunk[i * 4 + 1] = v2;
                                *vert_chunk[i * 4 + 2] = v3;
                                *vert_chunk[i * 4 + 3] = v4;
                            }
                        });
                }
            }

            #[cfg(debug_assertions)]
            {
                // Sanity check: verify we wrote all the vertices
                // @Robustness: should be asserting equality, but Vertex has no PartialEq impl.
                fn is_invalid_vertex(v: &Vertex) -> bool {
                    let nv = invalid_vertex();
                    v.position == nv.position
                        && v.color == nv.color
                        && v.tex_coords == nv.tex_coords
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
                let shadow_vbuffer = shadow_vbuffer.as_mut().unwrap();
                shadow_vbuffer.update(shadow_vertices, n_shadow_vertices);

                backend::render_vbuf_ws_ex(
                    window,
                    &shadow_vbuffer.vbuf,
                    &Transform2D::default(),
                    camera,
                    render::Render_Extra_Params {
                        texture: Some(texture),
                        ..Default::default()
                    },
                );
            }

            vbuffer.update(vertices, n_vertices_without_shadows);

            let shader = shader.map(|s| s as &_);
            backend::render_vbuf_ws_ex(
                window,
                &vbuffer.vbuf,
                &Transform2D::default(),
                camera,
                render::Render_Extra_Params {
                    texture: Some(texture),
                    shader,
                },
            );
        }
    }
}
