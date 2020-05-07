use crate::alloc::temp;
use crate::cfg::{self, Cfg_Var};
use crate::common::angle::Angle;
use crate::common::colors::{self, Color};
use crate::common::rect::Rect;
use crate::common::transform::Transform2D;
use crate::common::vector::Vec2f;
use crate::ecs::components::gfx::Material;
use crate::gfx::light::Lights;
use crate::gfx::render::{self, Shader, Texture, Vertex};
use crate::gfx::window::Window_Handle;
use crate::resources::gfx::{Gfx_Resources, Shader_Cache, Texture_Handle};
use crate::common::math::lerp;
use rayon::prelude::*;
use std::cmp;
use std::collections::{BTreeMap, HashMap};

// @Cleanup
type Vertex_Buffer = sfml::graphics::VertexBuffer;
use sfml::graphics::PrimitiveType;
use sfml::graphics::VertexBufferUsage;
use sfml::graphics::RenderTarget;

#[derive(Default)]
pub struct Batches {
    textures_ws:
        BTreeMap<render::Z_Index, HashMap<Material, (Vertex_Buffer_Holder, Vec<Texture_Props_Ws>)>>,
}

struct Vertex_Buffer_Holder {
    pub vbuf: Vertex_Buffer,
    pub n_elems: u32,
    #[cfg(debug_assertions)]
    id: Material,
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

        // @WaitForStable make this const
        let null_vertex: Vertex =
            Vertex::new(v2!(0., 0.), colors::TRANSPARENT.into(), v2!(0., 0.).into());

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
            *vertex = null_vertex;
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
                (
                    Vertex_Buffer_Holder::with_initial_vertex_count(
                        48,
                        #[cfg(debug_assertions)]
                        material,
                    ),
                    vec![],
                )
            })
            .1
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
        .for_each(|m| m.values_mut().for_each(|(_, v)| v.clear()));
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
    const MAX_ENCODED: u32 = 65535;
    let rad = rot.as_rad_0tau();
    let encoded = (rad * MAX_ENCODED as f32 / std::f32::consts::PI * 0.5) as u32;
    Color {
        r: 0,
        g: 0,
        b: ((encoded >> 8) & 0xFF) as u8,
        a: (encoded & 0xFF) as u8,
    }
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
    shadows: HashMap<Texture_Handle, Vec<(Transform2D, Rect<i32>)>>,
) {
    trace!("draw_all_batches");

    let enable_shaders = Cfg_Var::<bool>::new("engine/rendering/enable_shaders", cfg).read(cfg);

    // for each Z-index...
    for (z_index, tex_map) in &mut batches.textures_ws {
        // for each texture/shader...
        for (material, (vbuffer, tex_props)) in tex_map {
            let n_texs = tex_props.len(); 
            if n_texs == 0 {
                // @Speed: right now we don't delete this batch from the tex_map, but it may be worth doing so.
                continue;
            }

            let texture = gres.get_texture(material.texture);

            // @Temporary
            let shader = if enable_shaders {
                material.shader.map(|id| {
                    let shader = shader_cache.get_shader_mut(Some(id));
                    if material.normals.is_some() {
                        unsafe {
                            set_uniform_texture_workaround(
                                shader,
                                gres,
                                "normals",
                                material.normals,
                            );
                        }
                    }
                    fn col2v3(color: Color) -> sfml::graphics::glsl::Vec3 {
                        let c =
                            sfml::graphics::glsl::Vec4::from(sfml::graphics::Color::from(color));
                        sfml::graphics::glsl::Vec3::new(c.x, c.y, c.z)
                    }
                    shader.set_uniform_vec3(
                        "ambient_light.color",
                        col2v3(lights.ambient_light.color),
                    );
                    shader.set_uniform_float(
                        "ambient_light.intensity",
                        lights.ambient_light.intensity,
                    );
                    shader.set_uniform_current_texture("texture");
                    for (i, pl) in lights.point_lights.iter().enumerate() {
                        shader.set_uniform_vec2(
                            &format!("point_lights[{}].position", i),
                            sfml::graphics::glsl::Vec2::new(pl.position.x, pl.position.y),
                        );
                        shader.set_uniform_vec3(
                            &format!("point_lights[{}].color", i),
                            col2v3(pl.color),
                        );
                        shader.set_uniform_float(&format!("point_lights[{}].radius", i), pl.radius);
                        shader.set_uniform_float(
                            &format!("point_lights[{}].attenuation", i),
                            pl.attenuation,
                        );
                    }
                    shader.set_uniform_float(
                        "shininess",
                        Material::decode_shininess(material.shininess),
                    );
                    shader.set_uniform_vec3("specular_color", col2v3(material.specular_color));
                    shader
                })
            } else {
                None
            };

            let cast_shadows = *z_index >= 0; // @Temporary
            let n_threads = rayon::current_num_threads();
            let n_texs_per_chunk = cmp::min(n_texs, n_texs / n_threads + 1);

            debug_assert!(n_texs * 4 * if cast_shadows { 2 } else { 1 }<= std::u32::MAX as usize);
            let n_vertices_without_shadows = (n_texs * 4) as u32;
            let n_vertices = n_vertices_without_shadows * if cast_shadows { 2 } else { 1 };

            let mut vertices = temp::excl_temp_array(frame_alloc);
            unsafe {
                // Note: we allocate extra space if vbuffer.n_elems > n_vertices (i.e. the number
                // of actual vertices we're gonna add) because we'll use it to overwrite the
                // current buffer with "null" vertices.
                vertices.alloc_additional_uninit(n_vertices.max(vbuffer.n_elems) as usize);
            }
            let (vertices, shadow_vertices) = vertices.split_at_mut(n_vertices_without_shadows as usize);
            let vert_chunks = vertices[..n_vertices_without_shadows as usize]
                .par_iter_mut()
                .chunks(n_texs_per_chunk * 4);

            if n_vertices > vbuffer.vbuf.vertex_count() {
                vbuffer.grow(n_vertices);
            }
            debug_assert!(n_vertices <= vbuffer.vbuf.vertex_count());

            let has_shader = shader.is_some();
            {
                trace!("tex_batch_ws");
                if cast_shadows {
                    let shadow_chunks = shadow_vertices[..(n_vertices - n_vertices_without_shadows) as usize].par_iter_mut().chunks(n_texs_per_chunk * 4);
                    tex_props
                        .par_iter()
                        .chunks(n_texs_per_chunk)
                        .zip(vert_chunks)
                        .zip(shadow_chunks)
                        .for_each(|((tex_chunk, mut vert_chunk), mut shad_chunk)| {
                            for (i, tex_prop) in tex_chunk.iter().enumerate() {
                                let Texture_Props_Ws {
                                    tex_rect,
                                    transform,
                                    color,
                                } = tex_prop;

                                let uv: Rect<f32> = (*tex_rect).into();
                                let tex_size = Vec2f::new(tex_rect.width as _, tex_rect.height as _);
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

                                let mut v1 = render::new_vertex(p1, color, v2!(uv.x, uv.y));
                                let mut v2 = render::new_vertex(p2, color, v2!(uv.x + uv.width, uv.y));
                                let mut v3 = render::new_vertex(
                                    p3,
                                    color,
                                    v2!(uv.x + uv.width, uv.y + uv.height),
                                );
                                let mut v4 = render::new_vertex(p4, color, v2!(uv.x, uv.y + uv.height));

                                *vert_chunk[i * 4] = v1;
                                *vert_chunk[i * 4 + 1] = v2;
                                *vert_chunk[i * 4 + 2] = v3;
                                *vert_chunk[i * 4 + 3] = v4;

                                // @Incomplete:
                                // we should be retrieving the nearest light position by querying the world.
                                // Should we support multiple shadows per entity? In that case, they should be
                                // probably calculated beforehand
                                let light_pos = v2!(0., 0.);
                                let d1 = light_pos - p1;
                                let d2 = light_pos - p2;
                                let d3 = light_pos - p3;
                                let d4 = light_pos - p4;

                                let v = [d1, d2, d3, d4];

                                let max_d_sqr = v.iter().map(|d| 
                                    d.magnitude2()
                                ).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

                                const SHADOW_COLOR: sfml::graphics::Color = sfml::graphics::Color { r: 0, g: 0, b: 0, a: 150 };

                                v1.position -= (lerp(0.0, 1.0, d1.magnitude2() / max_d_sqr - 1.0) * d1).into();
                                v1.color = SHADOW_COLOR;
                                v2.position -= (lerp(0.0, 1.0, d2.magnitude2() / max_d_sqr - 1.0) * d2).into();
                                v2.color = SHADOW_COLOR;
                                v3.position -= (lerp(0.0, 1.0, d3.magnitude2() / max_d_sqr - 1.0) * d3).into();
                                v3.color = SHADOW_COLOR;
                                v4.position -= (lerp(0.0, 1.0, d4.magnitude2() / max_d_sqr - 1.0) * d4).into();
                                v4.color = SHADOW_COLOR;

                                *shad_chunk[i * 4] = v1;
                                *shad_chunk[i * 4 + 1] = v2;
                                *shad_chunk[i * 4 + 2] = v3;
                                *shad_chunk[i * 4 + 3] = v4;
                            }
                        });
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
                                let tex_size = Vec2f::new(tex_rect.width as _, tex_rect.height as _);
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

            if cast_shadows {
                vbuffer.update(shadow_vertices, n_vertices - n_vertices_without_shadows);

                // @Temporary
                window.raw_handle_mut().draw_vertex_buffer(
                    &vbuffer.vbuf,
                    sfml::graphics::RenderStates {
                        texture: Some(texture),
                        transform: camera.get_matrix_sfml().inverse(),
                        ..sfml::graphics::RenderStates::default()
                    },
                );
            }

            vbuffer.update(vertices, n_vertices);

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
