use crate::light::{Lights, Point_Light, Rect_Light};
use crate::material::Material;
use crate::render::{self, Primitive_Type};
use crate::vbuf_holder::Vertex_Buffer_Holder;
use inle_alloc::temp;
use inle_alloc::temp::excl_temp_array::Exclusive_Temp_Array;
use inle_common::colors::{self, Color, Color3};
use inle_gfx_backend::render::get_vp_matrix;
use inle_gfx_backend::render::{Shader, Texture, Vertex};
use inle_gfx_backend::render_window::Render_Window_Handle;
use inle_math::angle::Angle;
use inle_math::math::{lerp, lerp_clamped};
use inle_math::matrix::Matrix3;
use inle_math::rect;
use inle_math::rect::{Rect, Rectf};
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use inle_resources::gfx::{Gfx_Resources, Shader_Cache};
use smallvec::SmallVec;
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;

const SHADOWS_PER_ENTITY: usize = 4;
const VERTICES_PER_SPRITE: usize = 6;
const MAX_POINT_LIGHTS: usize = 4;
const MAX_RECT_LIGHTS: usize = 4;

struct Sprite_Batch {
    pub vbuffer: Vertex_Buffer_Holder,
    pub shadow_vbuffer: Option<Vertex_Buffer_Holder>,
    pub sprites: Vec<Sprite>,
}

#[derive(Default)]
pub struct Batches {
    textures_ws: BTreeMap<super::Z_Index, HashMap<Material, Sprite_Batch>>,
    point_lights_near_camera: SmallVec<[Point_Light; MAX_POINT_LIGHTS]>,
    rect_lights_near_camera: SmallVec<[Rect_Light; MAX_RECT_LIGHTS]>,
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
    material: &Material,
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
            .entry(*material)
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

fn update_light_uniforms(
    ubo: &mut render::Uniform_Buffer,
    lights: &Lights,
    point_lights_near_camera: &[Point_Light],
    rect_lights_near_camera: &[Rect_Light],
) {
    trace!("update_light_uniforms");

    // Assuming this layout in GLSL:
    //
    // layout (std140) uniform LightsBlock {
    //     Ambient_Light ambient_light;
    //     Point_Light point_lights[MAX_POINT_LIGHTS];
    //     Rect_Light rect_lights[MAX_RECT_LIGHTS];
    // };

    let mut next_offset = 0;
    const COL_NORM: f32 = 1.0 / 255.0;

    {
        #[repr(C)]
        #[derive(Copy, Clone)]
        struct Ambient_Light {
            r: f32,
            g: f32,
            b: f32,
            intensity: f32,
        }
        unsafe impl render::Std140 for Ambient_Light {}

        let light = lights.ambient_light();
        let ambient_light = Ambient_Light {
            r: light.color.r as f32 * COL_NORM,
            b: light.color.b as f32 * COL_NORM,
            g: light.color.g as f32 * COL_NORM,
            intensity: light.intensity,
        };
        next_offset = render::write_into_uniform_buffer(ubo, next_offset, ambient_light);
    }

    {
        #[repr(C)]
        #[derive(Copy, Clone, Default)]
        struct Point_Light {
            r: f32,
            g: f32,
            b: f32,
            intensity: f32,
            position: Vec2f,
            radius: f32,
            attenuation: f32,
        }
        unsafe impl render::Std140 for Point_Light {}

        let mut point_lights = [Point_Light::default(); MAX_POINT_LIGHTS];
        for (i, pl) in point_lights_near_camera
            .iter()
            .take(MAX_POINT_LIGHTS)
            .enumerate()
        {
            let point_light = Point_Light {
                r: pl.color.r as f32 * COL_NORM,
                g: pl.color.g as f32 * COL_NORM,
                b: pl.color.b as f32 * COL_NORM,
                intensity: pl.intensity,
                position: pl.position,
                radius: pl.radius,
                attenuation: pl.attenuation,
            };
            point_lights[i] = point_light;
        }

        next_offset = render::write_array_into_uniform_buffer(ubo, next_offset, &point_lights);
    }

    {
        #[repr(C)]
        #[derive(Copy, Clone, Default)]
        struct Rect_Light {
            r: f32,
            g: f32,
            b: f32,
            intensity: f32,
            pos_min: Vec2f,
            pos_max: Vec2f,
            radius: f32,
            attenuation: f32,
            _pad1: f32,
            _pad2: f32,
        }
        unsafe impl render::Std140 for Rect_Light {}

        let mut rect_lights = [Rect_Light::default(); MAX_RECT_LIGHTS];
        for (i, rl) in rect_lights_near_camera
            .iter()
            .take(MAX_RECT_LIGHTS)
            .enumerate()
        {
            let rect_light = Rect_Light {
                r: rl.color.r as f32 * COL_NORM,
                g: rl.color.g as f32 * COL_NORM,
                b: rl.color.b as f32 * COL_NORM,
                intensity: rl.intensity,
                pos_min: rl.rect.pos_min(),
                pos_max: rl.rect.pos_max(),
                radius: rl.radius,
                attenuation: rl.attenuation,
                _pad1: 0.0,
                _pad2: 0.0,
            };
            rect_lights[i] = rect_light;
        }

        render::write_array_into_uniform_buffer(ubo, next_offset, &rect_lights);
    }
}

fn set_shader_uniforms(
    window: &mut Render_Window_Handle,
    shader: &mut Shader,
    material: &Material,
    gres: &Gfx_Resources,
    lights: &Lights,
    point_lights_near_camera: &[Point_Light],
    rect_lights_near_camera: &[Rect_Light],
    lights_ubo_needs_update: bool,
    texture: &Texture,
    view_projection: &Matrix3<f32>,
) {
    trace!("set_shader_uniforms");

    use super::set_uniform;

    super::use_shader(shader);

    // @TODO: use UBOs for all uniforms

    set_uniform(shader, c_str!("tex"), texture);
    set_uniform(shader, c_str!("vp"), view_projection);

    if material.normals.is_some() {
        let normals = gres.get_texture(material.normals);
        set_uniform(shader, c_str!("normals"), normals);
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

    let lights_ubo = render::create_or_get_uniform_buffer(window, shader, c_str!("LightsBlock"));
    // Note: we only update the light uniforms if lights have changed AND we didn't update this particular UBO yet.
    if lights_ubo_needs_update && !render::uniform_buffer_needs_transfer_to_gpu(lights_ubo) {
        update_light_uniforms(
            lights_ubo,
            lights,
            point_lights_near_camera,
            rect_lights_near_camera,
        );
    }
    render::bind_uniform_buffer(lights_ubo);
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
    lights: &mut Lights,
    draw_params: Batcher_Draw_Params,
    frame_alloc: &mut temp::Temp_Allocator,
) {
    trace!("draw_all_batches");

    let view_projection = get_vp_matrix(window, camera);
    let visible_viewport = inle_win::window::get_camera_viewport(window, camera);

    let mut lights_ubo_needs_update = lights.process_commands();

    {
        let mut old_point_lights_near_camera = SmallVec::<[Point_Light; MAX_POINT_LIGHTS]>::new();
        std::mem::swap(
            &mut old_point_lights_near_camera,
            &mut batches.point_lights_near_camera,
        );
        // @Speed: should lights be spatially accelerated?
        lights.get_all_point_lights_sorted_by_distance_within(
            camera.position(),
            visible_viewport.width * 2.0,
            &mut batches.point_lights_near_camera,
            MAX_POINT_LIGHTS,
        );
        batches
            .point_lights_near_camera
            .resize(MAX_POINT_LIGHTS, Point_Light::default());
        lights_ubo_needs_update = lights_ubo_needs_update
            || batches.point_lights_near_camera != old_point_lights_near_camera;

        let mut old_rect_lights_near_camera = SmallVec::<[Rect_Light; MAX_RECT_LIGHTS]>::new();
        std::mem::swap(
            &mut old_rect_lights_near_camera,
            &mut batches.rect_lights_near_camera,
        );
        // @Speed: should lights be spatially accelerated?
        lights.get_all_rect_lights_sorted_by_distance_within(
            camera.position(),
            visible_viewport.width * 2.0,
            &mut batches.rect_lights_near_camera,
            MAX_POINT_LIGHTS,
        );
        batches
            .rect_lights_near_camera
            .resize(MAX_POINT_LIGHTS, Rect_Light::default());
        lights_ubo_needs_update = lights_ubo_needs_update
            || batches.rect_lights_near_camera != old_rect_lights_near_camera;
    }

    let point_lights_near_camera = &batches.point_lights_near_camera;
    let rect_lights_near_camera = &batches.rect_lights_near_camera;

    // for each Z-index...
    for sprite_map in batches.textures_ws.values_mut() {
        // for each material...
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
                    set_shader_uniforms(
                        window,
                        shader,
                        material,
                        gres,
                        lights,
                        point_lights_near_camera,
                        rect_lights_near_camera,
                        lights_ubo_needs_update,
                        texture,
                        &view_projection,
                    );
                    shader
                })
            } else {
                None
            };
            let has_shader = shader.is_some();
            let cast_shadows = draw_params.enable_shadows && material.cast_shadows;

            // This buffer holds both regular vertices and shadow vertices
            let mut vertices = temp::excl_temp_array(frame_alloc);

            let n_vertices;
            let n_shadow_vertices;
            if cast_shadows {
                let (nv, ns) = fill_vertices_with_shadows(
                    texture,
                    sprites,
                    &visible_viewport,
                    lights,
                    has_shader,
                    &mut vertices,
                );
                n_vertices = nv;
                n_shadow_vertices = ns;
                debug_assert_eq!(vertices.len(), nv + ns);
            } else {
                n_vertices = fill_vertices(
                    texture,
                    sprites,
                    &visible_viewport,
                    has_shader,
                    &mut vertices,
                );
                n_shadow_vertices = 0;
            }
            let n_vertices = u32::try_from(n_vertices).unwrap();
            let n_shadow_vertices = u32::try_from(n_shadow_vertices).unwrap();

            debug_assert!(!((n_shadow_vertices > 0) && !cast_shadows));

            // Ensure the vbuffer has enough room to write in
            if n_vertices > super::vbuf_max_vertices(&vbuffer.vbuf) {
                vbuffer.grow(window, n_vertices);
            }

            if n_shadow_vertices > 0 {
                // @Cleanup: we can probably get rid of shadow_vbuffer and just use vbuffer twice
                let shadow_vbuffer = shadow_vbuffer.as_mut().unwrap();
                if n_shadow_vertices > super::vbuf_max_vertices(&shadow_vbuffer.vbuf) {
                    shadow_vbuffer.grow(window, n_shadow_vertices);
                }
                shadow_vbuffer.update(&vertices, n_shadow_vertices);

                render::render_vbuf_ws_with_texture(
                    window,
                    &shadow_vbuffer.vbuf,
                    &Transform2D::default(),
                    camera,
                    texture,
                );
            }

            vbuffer.update(
                &vertices.as_slice()[n_shadow_vertices as usize..],
                n_vertices,
            );

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

fn fill_vertices_with_shadows(
    texture: &Texture,
    sprites: &[Sprite],
    visible_viewport: &Rectf,
    lights: &Lights,
    has_shader: bool,
    out_vertices: &mut Exclusive_Temp_Array<Vertex>,
) -> (usize, usize) {
    trace!("fill_vertices_with_shadows");

    let n_shadow_vertices =
        fill_shadow_vertices(texture, sprites, visible_viewport, lights, out_vertices);
    let n_sprite_vertices =
        fill_vertices(texture, sprites, visible_viewport, has_shader, out_vertices);

    (n_sprite_vertices, n_shadow_vertices)
}

fn fill_vertices(
    texture: &Texture,
    sprites: &[Sprite],
    visible_viewport: &Rectf,
    has_shader: bool,
    out_vertices: &mut Exclusive_Temp_Array<Vertex>,
) -> usize {
    trace!("fill_vertices");

    let tex_size = render::get_texture_size(texture);
    let (tw, th) = (tex_size.0 as f32, tex_size.1 as f32);
    let mut n_vertices_added = 0;

    for sprite in sprites {
        let Sprite {
            tex_rect,
            transform,
            color,
        } = sprite;

        let uv = Rect::new(
            tex_rect.x as f32 / tw,
            tex_rect.y as f32 / th,
            tex_rect.width as f32 / tw,
            tex_rect.height as f32 / th,
        );
        let sprite_size = v2!(tex_rect.width as f32, tex_rect.height as f32);
        let render_transform = *transform;

        // Note: beware of the order of multiplications!
        // Scaling the local positions must be done BEFORE multiplying the matrix!
        let p1 = render_transform * (sprite_size * v2!(-0.5, -0.5));
        let p2 = render_transform * (sprite_size * v2!(0.5, -0.5));
        let p3 = render_transform * (sprite_size * v2!(0.5, 0.5));
        let p4 = render_transform * (sprite_size * v2!(-0.5, 0.5));

        let sprite_aabb = rect::aabb_of_points(&[p1, p2, p3, p4]);
        if rect::rects_intersection(visible_viewport, &sprite_aabb).is_none() {
            continue;
        }

        // Encode rotation in color
        let color = if has_shader {
            encode_rot_and_alpha_as_color(transform.rotation(), color.a)
        } else {
            *color
        };

        let v1 = super::new_vertex(p1, color, v2!(uv.x, uv.y));
        let v2 = super::new_vertex(p2, color, v2!(uv.x + uv.width, uv.y));
        let v3 = super::new_vertex(p3, color, v2!(uv.x + uv.width, uv.y + uv.height));
        let v4 = super::new_vertex(p4, color, v2!(uv.x, uv.y + uv.height));

        out_vertices.push(v1);
        out_vertices.push(v2);
        out_vertices.push(v3);
        out_vertices.push(v3);
        out_vertices.push(v4);
        out_vertices.push(v1);

        n_vertices_added += 6;
    }

    n_vertices_added
}

fn fill_shadow_vertices(
    texture: &Texture,
    sprites: &[Sprite],
    visible_viewport: &Rectf,
    lights: &Lights,
    out_vertices: &mut Exclusive_Temp_Array<Vertex>,
) -> usize {
    // @Speed: duplicate calculation
    let tex_size = render::get_texture_size(texture);
    let (tw, th) = (tex_size.0 as f32, tex_size.1 as f32);

    let mut n_vertices_added = 0;

    for sprite in sprites {
        // @Speed: calculating v1, v2, v3 and v4 is something we're doing twice per sprite in the case of
        // materials that cast shadows. We should calculate them only once and save the results.
        let Sprite {
            tex_rect,
            transform,
            ..
        } = sprite;

        let uv = Rect::new(
            tex_rect.x as f32 / tw,
            tex_rect.y as f32 / th,
            tex_rect.width as f32 / tw,
            tex_rect.height as f32 / th,
        );
        let sprite_size = v2!(tex_rect.width as f32, tex_rect.height as f32);
        let render_transform = *transform;
        let p1 = render_transform * (sprite_size * v2!(-0.5, -0.5));
        let p2 = render_transform * (sprite_size * v2!(0.5, -0.5));
        let p3 = render_transform * (sprite_size * v2!(0.5, 0.5));
        let p4 = render_transform * (sprite_size * v2!(-0.5, 0.5));

        let sprite_aabb = rect::aabb_of_points(&[p1, p2, p3, p4]);
        if rect::rects_intersection(visible_viewport, &sprite_aabb).is_none() {
            continue;
        }

        let v1 = super::new_vertex(p1, colors::BLACK, v2!(uv.x, uv.y));
        let v2 = super::new_vertex(p2, colors::BLACK, v2!(uv.x + uv.width, uv.y));
        let v3 = super::new_vertex(p3, colors::BLACK, v2!(uv.x + uv.width, uv.y + uv.height));
        let v4 = super::new_vertex(p4, colors::BLACK, v2!(uv.x, uv.y + uv.height));

        let mut nearby_point_lights = SmallVec::<[Point_Light; SHADOWS_PER_ENTITY]>::new();
        // @Speed: should lights be spatially accelerated?
        lights.get_all_point_lights_sorted_by_distance_within(
            render_transform.position(),
            10000.,
            &mut nearby_point_lights,
            SHADOWS_PER_ENTITY,
        );
        debug_assert!(nearby_point_lights.len() <= SHADOWS_PER_ENTITY);
        nearby_point_lights.resize(SHADOWS_PER_ENTITY, Point_Light::default());

        for (light_idx, light) in nearby_point_lights.iter().enumerate() {
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

            const SHADOW_MAX_VALUE: f32 = 50.0;
            const SHADOW_MAX_DIST: f32 = 100.0;

            for v_idx in 0..v.len() {
                if v_idx <= 1 {
                    let offset =
                        lerp_clamped(0., SHADOW_MAX_DIST, 1.0 / (dist2[v_idx].powf(0.6) + 0.1));
                    v[v_idx].set_position(v[v_idx].position() - offset * diff[v_idx]);
                }

                let t = (1.0 - dist2[v_idx] * recp_radius2).max(0.0);
                v[v_idx].set_color(colors::rgba(
                    0,
                    0,
                    0,
                    lerp(0.0, SHADOW_MAX_VALUE, t * t) as u8,
                ));
                //match v_idx {
                //0 => v[v_idx].set_color(colors::rgba(255, 0, 0, 100)),
                //1 => v[v_idx].set_color(colors::rgba(0, 255, 0, 100)),
                //2 => v[v_idx].set_color(colors::rgba(0, 0, 255, 100)),
                //3 => v[v_idx].set_color(colors::rgba(255, 255, 0, 100)),
                //_ => unreachable!(),
                //}
            }
            let ps = [
                v[0].position(),
                v[1].position(),
                v[2].position(),
                v[3].position(),
            ];
            let shadow_aabb = rect::aabb_of_points(&ps);
            if rect::rects_intersection(visible_viewport, &shadow_aabb).is_none() {
                continue;
            }

            // Fix vertex winding, since it may now be wrong  after moving them
            let a = ps[1] - ps[0];
            let b = ps[2] - ps[0];
            let a_wedge_b = a.x * b.y - a.y * b.x;
            if a_wedge_b.signum() < 0.0 {
                v.swap(0, 1);
                v.swap(2, 3);
            }

            out_vertices.push(v[0]);
            out_vertices.push(v[1]);
            out_vertices.push(v[2]);
            out_vertices.push(v[2]);
            out_vertices.push(v[3]);
            out_vertices.push(v[0]);

            n_vertices_added += 6;
        }
    }

    n_vertices_added
}
