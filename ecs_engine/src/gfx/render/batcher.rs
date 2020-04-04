use crate::alloc::temp;
use crate::common::colors::{self, Color};
use crate::common::rect::Rect;
use crate::common::transform::Transform2D;
use crate::common::vector::Vec2f;
use crate::gfx::render;
use crate::gfx::window::Window_Handle;
use crate::resources::gfx::{Gfx_Resources, Texture_Handle};
use rayon::prelude::*;
use std::cmp;
use std::collections::{BTreeMap, HashMap};

type Vertex_Buffer = sfml::graphics::VertexBuffer;
use sfml::graphics::PrimitiveType;
use sfml::graphics::VertexBufferUsage;

#[derive(Default)]
pub struct Batches {
    textures_ws: BTreeMap<
        render::Z_Index,
        HashMap<Texture_Handle, (Vertex_Buffer_Holder, Vec<Texture_Props_Ws>)>,
    >,
}

struct Vertex_Buffer_Holder {
    pub vbuf: Vertex_Buffer,
    pub n_elems: usize,
}

struct Texture_Props_Ws {
    pub tex_rect: Rect<i32>,
    pub color: Color,
    pub transform: Transform2D,
}

pub(super) fn add_texture_ws(
    batches: &mut Batches,
    texture: Texture_Handle,
    tex_rect: &Rect<i32>,
    color: Color,
    transform: &Transform2D,
    z_index: render::Z_Index,
) {
    batches
        .textures_ws
        .entry(z_index)
        .or_insert_with(HashMap::new)
        .entry(texture)
        .or_insert_with(|| {
            ldebug!("creating buffer for texture {:?}", texture);
            (
                // @Incomplete: make a growable vertex buffer
                Vertex_Buffer_Holder {
                    vbuf: Vertex_Buffer::new(
                        PrimitiveType::Quads,
                        65536,
                        VertexBufferUsage::Stream,
                    ),
                    n_elems: 0,
                },
                vec![],
            )
        })
        .1
        .push(Texture_Props_Ws {
            tex_rect: *tex_rect,
            color,
            transform: *transform,
        });
}

pub fn clear_batches(batches: &mut Batches) {
    trace!("clear_batches");
    batches
        .textures_ws
        .values_mut()
        .for_each(|m| m.values_mut().for_each(|(_, v)| v.clear()));
}

pub fn draw_batches(
    window: &mut Window_Handle,
    gres: &Gfx_Resources,
    batches: &mut Batches,
    camera: &Transform2D,
    frame_alloc: &mut temp::Temp_Allocator,
) {
    trace!("draw_all_batches");

    // for each Z-index...
    for tex_map in batches.textures_ws.values_mut() {
        // for each texture...
        for (tex_id, (vbuffer, tex_props)) in tex_map {
            let texture = gres.get_texture(*tex_id);

            let n_threads = rayon::current_num_threads();
            let n_texs = tex_props.len();
            let n_texs_per_chunk = cmp::min(n_texs, n_texs / n_threads + 1);
            let n_vertices = n_texs * 4;

            let mut vertices = temp::excl_temp_array(frame_alloc);
            unsafe {
                // Note: we allocate extra space if vbuffer.n_elems > n_vertices (i.e. the number
                // of actual vertices we're gonna add) because we'll use it to overwrite the
                // current buffer with "null" vertices.
                vertices.alloc_additional_uninit(n_vertices.max(vbuffer.n_elems));
            }
            let vertices = vertices.as_slice_mut();
            let vert_chunks = vertices[..n_vertices]
                .par_iter_mut()
                .chunks(n_texs_per_chunk * 4);

            debug_assert!(n_vertices <= vbuffer.vbuf.vertex_count() as usize);

            {
                trace!("tex_batch_ws");
                tex_props
                    .par_iter()
                    .chunks(n_texs_per_chunk)
                    .zip(vert_chunks)
                    .for_each(|(tex_chunk, mut vert_chunk)| {
                        for (i, tex_prop) in tex_chunk.iter().enumerate() {
                            let Texture_Props_Ws {
                                tex_rect,
                                color,
                                transform,
                            } = tex_prop;

                            let color = *color;
                            let uv: Rect<f32> = (*tex_rect).into();
                            let tex_size = Vec2f::new(tex_rect.width as _, tex_rect.height as _);
                            let render_transform = *transform;

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

            {
                trace!("vbuf_update");

                // @WaitForStable make this const
                let null_vertex = render::Vertex::new(
                    v2!(0., 0.),
                    colors::TRANSPARENT.into(),
                    v2!(0., 0.).into(),
                );

                for vertex in vertices.iter_mut().take(vbuffer.n_elems).skip(n_vertices) {
                    *vertex = null_vertex;
                }
                vbuffer.vbuf.update(vertices, 0);
                vbuffer.n_elems = vertices.len();
            }

            // @Temporary
            use sfml::graphics::RenderTarget;
            window.raw_handle_mut().draw_vertex_buffer(
                &vbuffer.vbuf,
                sfml::graphics::RenderStates {
                    texture: Some(texture),
                    transform: camera.get_matrix_sfml().inverse(),
                    ..sfml::graphics::RenderStates::default()
                },
            );
        }
    }
}
