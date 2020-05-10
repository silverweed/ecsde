use crate::alloc::temp;
use crate::cfg::{self, Cfg_Var};
use crate::common::angle::Angle;
use crate::common::colors::{self, Color};
use crate::common::math::{lerp, lerp_clamped};
use crate::common::rect::Rect;
use crate::common::transform::Transform2D;
use crate::common::vector::Vec2f;
use crate::ecs::components::gfx::Material;
use crate::gfx::light::Lights;
use crate::gfx::render::{self, Shader, Texture, Vertex};
use crate::gfx::window::Window_Handle;
use crate::resources::gfx::{Gfx_Resources, Shader_Cache, Texture_Handle};
use rayon::prelude::*;
use std::cmp;
use std::collections::{BTreeMap, HashMap};

// @Cleanup
type Vertex_Buffer = sfml::graphics::VertexBuffer;
use sfml::graphics::PrimitiveType;
use sfml::graphics::RenderTarget;
use sfml::graphics::VertexBufferUsage;

struct Texture_Batch {
    pub vbuffer: Vertex_Buffer_Holder,
    pub shadow_vbuffer: Option<Vertex_Buffer_Holder>,
    pub tex_props: Vec<Texture_Props_Ws>,
}

#[derive(Default)]
pub struct Batches {
    textures_ws: BTreeMap<render::Z_Index, HashMap<Material, Texture_Batch>>,
}

struct Vertex_Buffer_Holder {
    pub vbuf: Vertex_Buffer,
    pub n_elems: u32,
    #[cfg(debug_assertions)]
    id: Material,
}

// @WaitForStable make this const
fn null_vertex() -> Vertex {
    Vertex::new(v2!(0., 0.), colors::TRANSPARENT.into(), v2!(0., 0.).into())
}

// @WaitForStable make this const
#[cfg(debug_assertions)]
fn invalid_vertex() -> Vertex {
    Vertex::new(
        v2!(-12345.6789, 9876.54321),
        colors::rgba(42, 42, 42, 42).into(),
        v2!(42., 42.).into(),
    )
}

impl Vertex_Buffer_Holder {
    pub fn with_initial_vertex_count(
        initial_cap: u32,
        #[cfg(debug_assertions)] id: Material,
    ) -> Self {
        Self {
            vbuf: Vertex_Buffer::new(PrimitiveType::Quads, initial_cap, VertexBufferUsage::Stream),
            n_elems: 0,
            #[cfg(debug_assertions)]
            id,
        }
    }

    pub fn update(&mut self, vertices: &mut [Vertex], n_vertices: u32) {
        trace!("vbuf_update");

        debug_assert!(vertices.len() <= std::u32::MAX as usize);

        debug_assert!(
            n_vertices <= self.vbuf.vertex_count(),
            "Can't hold all the vertices! {} / {}",
            n_vertices,
            self.vbuf.vertex_count()
        );

        // Zero all the excess vertices
        for vertex in vertices
            .iter_mut()
            .take(self.n_elems as usize)
            .skip(n_vertices as usize)
        {
            *vertex = null_vertex();
        }

        self.vbuf.update(vertices, 0);
        self.n_elems = vertices.len() as u32;
    }

    pub fn grow(&mut self, vertices_to_hold_at_least: u32) {
        let new_cap = vertices_to_hold_at_least.next_power_of_two();
        ldebug!(
            "Growing Vertex_Buffer_Holder {:?} to hold {} vertices ({} requested).",
            self.id,
            new_cap,
            vertices_to_hold_at_least
        );

        let mut new_vbuf =
            Vertex_Buffer::new(PrimitiveType::Quads, new_cap, VertexBufferUsage::Stream);
        let _res = new_vbuf.update_from_vertex_buffer(&self.vbuf);
        #[cfg(debug_assertions)]
        {
            debug_assert!(_res, "Vertex Buffer copying failed ({:?})!", self.id);
        }

        self.vbuf = new_vbuf;
    }
}

struct Texture_Props_Ws {
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

    let tex_batches = {
        trace!("get_tex_batches");
        &mut z_index_texmap
            .entry(material)
            .or_insert_with(|| {
                ldebug!("creating buffer for material {:?}", material,);
                Texture_Batch {
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
                    tex_props: vec![],
                }
            })
            .tex_props
    };

    {
        trace!("push_tex");
        tex_batches.push(Texture_Props_Ws {
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
        .for_each(|m| m.values_mut().for_each(|batch| batch.tex_props.clear()));
}

// !!! @Hack !!! to make set_uniform_texture work until https://github.com/jeremyletang/rust-sfml/issues/213 is solved
#[allow(unused_unsafe)]
unsafe fn set_uniform_texture_workaround(
    shader: &mut Shader,
    gres: &Gfx_Resources,
    name: &str,
    texture: Texture_Handle,
) {
    let tex = unsafe {
        std::mem::transmute::<&Texture, *const Texture<'static>>(gres.get_texture(texture))
    };
    shader.set_uniform_texture(name, unsafe { &*tex });
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
    use sfml::graphics::glsl;

    fn col2v3(color: Color) -> glsl::Vec3 {
        let c = glsl::Vec4::from(sfml::graphics::Color::from(color));
        glsl::Vec3::new(c.x, c.y, c.z)
    }

    if material.normals.is_some() {
        unsafe {
            set_uniform_texture_workaround(shader, gres, "normals", material.normals);
        }
    }
    shader.set_uniform_vec3("ambient_light.color", col2v3(lights.ambient_light.color));
    shader.set_uniform_float("ambient_light.intensity", lights.ambient_light.intensity);
    shader.set_uniform_current_texture("texture");
    let (tex_w, tex_h) = render::get_texture_size(texture);
    shader.set_uniform_vec2("texture_size", glsl::Vec2::new(tex_w as f32, tex_h as f32));
    for (i, pl) in lights.point_lights.iter().enumerate() {
        shader.set_uniform_vec2(
            &format!("point_lights[{}].position", i),
            glsl::Vec2::new(pl.position.x, pl.position.y),
        );
        shader.set_uniform_vec3(&format!("point_lights[{}].color", i), col2v3(pl.color));
        shader.set_uniform_float(&format!("point_lights[{}].radius", i), pl.radius);
        shader.set_uniform_float(&format!("point_lights[{}].attenuation", i), pl.attenuation);
    }
    shader.set_uniform_float("shininess", Material::decode_shininess(material.shininess));
    shader.set_uniform_vec3("specular_color", col2v3(material.specular_color));
}

pub fn draw_batches(
    window: &mut Window_Handle,
    gres: &Gfx_Resources,
    batches: &mut Batches,
    shader_cache: &mut Shader_Cache,
    camera: &Transform2D,
    lights: &Lights,
    cfg: &cfg::Config,
    frame_alloc: &mut temp::Temp_Allocator,
) {
    trace!("draw_all_batches");

    const SHADOWS_PER_ENTITY: usize = 4;

    let enable_shaders = Cfg_Var::<bool>::new("engine/rendering/enable_shaders", cfg).read(cfg);

    // for each Z-index...
    for tex_map in batches.textures_ws.values_mut() {
        // for each texture/shader...
        for (material, batch) in tex_map {
            let vbuffer = &mut batch.vbuffer;
            let shadow_vbuffer = &mut batch.shadow_vbuffer;
            let tex_props = &mut batch.tex_props;

            let n_texs = tex_props.len();
            if n_texs == 0 {
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

            let cast_shadows = material.cast_shadows;
            // @Temporary
            // @Speed! We can't use the temp array as it's not Send! Maybe we should make it Send (at least a read-only version of it)?
            let mut shadows = vec![];
            if cast_shadows {
                for tex in tex_props.iter() {
                    let mut nearby_point_lights = Vec::with_capacity(SHADOWS_PER_ENTITY); // @Speed!

                    // @Speed: should lights be spatially accelerated?
                    lights.get_all_point_lights_sorted_by_distance_within(
                        tex.transform.position(),
                        10000.,
                        &mut nearby_point_lights,
                        SHADOWS_PER_ENTITY,
                    );
                    nearby_point_lights.resize(4, crate::gfx::light::Point_Light::default());
                    shadows.push(nearby_point_lights);
                }
            }

            let n_threads = rayon::current_num_threads();
            let n_texs_per_chunk = cmp::min(n_texs, n_texs / n_threads + 1);

            let n_vertices_without_shadows = (n_texs * 4) as u32;
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

            let n_vert_per_chunk = n_texs_per_chunk * 4;
            let vert_chunks = vertices.par_iter_mut().chunks(n_vert_per_chunk);

            // Ensure the vbuffer has enough room to write in
            if n_vertices_without_shadows > vbuffer.vbuf.vertex_count() {
                vbuffer.grow(n_vertices_without_shadows);
            }
            debug_assert!(n_vertices_without_shadows <= vbuffer.vbuf.vertex_count());

            let has_shader = shader.is_some();
            {
                trace!("tex_batch_ws");

                if cast_shadows {
                    let shadow_vbuffer = shadow_vbuffer.as_mut().unwrap();
                    if n_shadow_vertices > shadow_vbuffer.vbuf.vertex_count() {
                        shadow_vbuffer.grow(n_shadow_vertices);
                    }
                    debug_assert!(n_shadow_vertices <= shadow_vbuffer.vbuf.vertex_count());

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

                    let tex_chunks = tex_props.par_iter().chunks(n_texs_per_chunk);
                    debug_assert_eq!(tex_chunks.len(), vert_chunks.len());
                    debug_assert_eq!(tex_chunks.len(), shadow_chunks.len());

                    tex_chunks.zip(vert_chunks).zip(shadow_chunks).for_each(
                        |((tex_chunk, mut vert_chunk), mut shadow_chunk)| {
                            for (i, tex_prop) in tex_chunk.iter().enumerate() {
                                let Texture_Props_Ws {
                                    tex_rect,
                                    transform,
                                    color,
                                } = tex_prop;

                                let uv: Rect<f32> = (*tex_rect).into();
                                let tex_size =
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
                                let p1 = render_transform * (tex_size * v2!(-0.5, -0.5));
                                let p2 = render_transform * (tex_size * v2!(0.5, -0.5));
                                let p3 = render_transform * (tex_size * v2!(0.5, 0.5));
                                let p4 = render_transform * (tex_size * v2!(-0.5, 0.5));

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

                                for (light_idx, light) in shadows[i].iter().enumerate() {
                                    debug_assert!(light_idx < 4);
                                    let light_pos = light.position;
                                    let recp_radius2 = 1.0 / (light.radius * light.radius);
                                    let mut v1 = v1;
                                    let mut v2 = v2;
                                    let mut v3 = v3;
                                    let mut v4 = v4;
                                    let d1 = light_pos - p1;
                                    let d2 = light_pos - p2;
                                    let d3 = light_pos - p3;
                                    let d4 = light_pos - p4;

                                    let dist2: [f32; 4] = [
                                        d1.magnitude2(),
                                        d2.magnitude2(),
                                        d3.magnitude2(),
                                        d4.magnitude2(),
                                    ];
                                    let min_d_sqr = dist2
                                        .iter()
                                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                                        .unwrap();

                                    const SHADOW_MAX_VALUE: f32 = 50.0;
                                    v1.position -=
                                        (lerp_clamped(0.0, 1.0, dist2[0] / min_d_sqr - 1.0) * d1)
                                            .into();
                                    let t = (1.0 - dist2[0] * recp_radius2).max(0.0);
                                    v1.color = colors::rgba(
                                        0,
                                        0,
                                        0,
                                        lerp(0.0, SHADOW_MAX_VALUE, t * t) as u8,
                                    )
                                    .into();

                                    v2.position -=
                                        (lerp_clamped(0.0, 1.0, dist2[1] / min_d_sqr - 1.0) * d2)
                                            .into();
                                    let t = (1.0 - dist2[1] * recp_radius2).max(0.0);
                                    v2.color = colors::rgba(
                                        0,
                                        0,
                                        0,
                                        lerp(0.0, SHADOW_MAX_VALUE, t * t) as u8,
                                    )
                                    .into();

                                    v3.position -=
                                        (lerp_clamped(0.0, 1.0, dist2[2] / min_d_sqr - 1.0) * d3)
                                            .into();
                                    let t = (1.0 - dist2[2] * recp_radius2).max(0.0);
                                    v3.color = colors::rgba(
                                        0,
                                        0,
                                        0,
                                        lerp(0.0, SHADOW_MAX_VALUE, t * t) as u8,
                                    )
                                    .into();

                                    v4.position -=
                                        (lerp_clamped(0.0, 1.0, dist2[3] / min_d_sqr - 1.0) * d4)
                                            .into();
                                    let t = (1.0 - dist2[3] * recp_radius2).max(0.0);
                                    v4.color = colors::rgba(
                                        0,
                                        0,
                                        0,
                                        lerp(0.0, SHADOW_MAX_VALUE, t * t) as u8,
                                    )
                                    .into();

                                    *shadow_chunk[4 * (SHADOWS_PER_ENTITY * i + light_idx)] = v1;
                                    *shadow_chunk[4 * (SHADOWS_PER_ENTITY * i + light_idx) + 1] =
                                        v2;
                                    *shadow_chunk[4 * (SHADOWS_PER_ENTITY * i + light_idx) + 2] =
                                        v3;
                                    *shadow_chunk[4 * (SHADOWS_PER_ENTITY * i + light_idx) + 3] =
                                        v4;
                                }
                                #[cfg(debug_assertions)]
                                {
                                    for light_idx in shadows[i].len()..4 {
                                        *shadow_chunk[4 * (SHADOWS_PER_ENTITY * i + light_idx)] =
                                            null_vertex();
                                        *shadow_chunk
                                            [4 * (SHADOWS_PER_ENTITY * i + light_idx) + 1] =
                                            null_vertex();
                                        *shadow_chunk
                                            [4 * (SHADOWS_PER_ENTITY * i + light_idx) + 2] =
                                            null_vertex();
                                        *shadow_chunk
                                            [4 * (SHADOWS_PER_ENTITY * i + light_idx) + 3] =
                                            null_vertex();
                                    }
                                }
                            }
                        },
                    );
                } else {
                    tex_props
                        .par_iter()
                        .chunks(n_texs_per_chunk)
                        .zip(vert_chunks)
                        .for_each(|(tex_chunk, mut vert_chunk)| {
                            for (i, tex_prop) in tex_chunk.iter().enumerate() {
                                let Texture_Props_Ws {
                                    tex_rect,
                                    transform,
                                    color,
                                } = tex_prop;

                                let uv: Rect<f32> = (*tex_rect).into();
                                let tex_size =
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
                                let p1 = render_transform * (tex_size * v2!(-0.5, -0.5));
                                let p2 = render_transform * (tex_size * v2!(0.5, -0.5));
                                let p3 = render_transform * (tex_size * v2!(0.5, 0.5));
                                let p4 = render_transform * (tex_size * v2!(-0.5, 0.5));

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
                    //ldebug!(
                    //"shadow_vertices.len() = {}. Non-null = {}",
                    //shadow_vertices.len(),
                    //shadow_vertices.iter().filter(|v| v.color.a != 0).count()
                    //);
                    //for (i, vert) in shadow_vertices.iter().enumerate() {
                    //ldebug!("shadow_vert[{}] = {:?}", i, vert.color);
                    //}
                    for (i, vert) in shadow_vertices.iter().enumerate() {
                        debug_assert!(!is_invalid_vertex(vert), "Shadow vertex {} is invalid!", i);
                    }
                }
            }

            if cast_shadows {
                let shadow_vbuffer = shadow_vbuffer.as_mut().unwrap();
                shadow_vbuffer.update(shadow_vertices, n_shadow_vertices);

                // @Temporary
                window.raw_handle_mut().draw_vertex_buffer(
                    &shadow_vbuffer.vbuf,
                    sfml::graphics::RenderStates {
                        texture: Some(texture),
                        transform: camera.get_matrix_sfml().inverse(),
                        ..sfml::graphics::RenderStates::default()
                    },
                );
            }

            vbuffer.update(vertices, n_vertices_without_shadows);

            // @Temporary
            let shader = shader.map(|s| s as &_);
            window.raw_handle_mut().draw_vertex_buffer(
                &vbuffer.vbuf,
                sfml::graphics::RenderStates {
                    texture: Some(texture),
                    shader,
                    transform: camera.get_matrix_sfml().inverse(),
                    ..sfml::graphics::RenderStates::default()
                },
            );
        }
    }
}
