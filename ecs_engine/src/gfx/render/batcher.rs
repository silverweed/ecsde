use crate::common::colors::Color;
use crate::common::rect::Rect;
use crate::common::shapes::Circle;
use crate::common::transform::Transform2D;
use crate::common::vector::{Vec2f, Vec2u};
use crate::gfx::paint_props::Paint_Properties;
use crate::gfx::render;
use crate::gfx::window::Window_Handle;
use crate::resources::gfx::{Font_Handle, Gfx_Resources, Texture_Handle};
use crossbeam_utils::thread;
use std::collections::HashMap;

type Vertex_Buffer = sfml::graphics::VertexBuffer;
use sfml::graphics::PrimitiveType;
use sfml::graphics::VertexBufferUsage;

#[derive(Default)]
pub struct Batches {
    textures_ws: HashMap<Texture_Handle, (Vertex_Buffer, Vec<Texture_Props>)>,
    rects_ws: Vec<Rect_Props_Ws>,
    rects: Vec<Rect_Props>,
    texts_ws: Vec<Text_Props_Ws>,
    texts: Vec<Text_Props>,
    circles_ws: Vec<Circle_Props_Ws>,
    lines: Vec<Line_Props>,
}

pub(super) struct Line_Props {
    pub from: render::Vertex,
    pub to: render::Vertex,
}

pub(super) struct Texture_Props {
    pub tex_rect: Rect<i32>,
    pub color: Color,
    pub transform: Transform2D,
}

pub(super) struct Rect_Props_Ws {
    pub rect: Rect<f32>,
    pub paint_props: Paint_Properties,
    pub transform: Transform2D,
}

pub(super) struct Rect_Props {
    pub rect: Rect<f32>,
    pub paint_props: Paint_Properties,
}

pub(super) struct Circle_Props_Ws {
    pub circle: Circle,
    pub paint_props: Paint_Properties,
}

pub(super) struct Text_Props_Ws {
    pub string: String,
    pub font: Font_Handle,
    pub font_size: u16,
    pub paint_props: Paint_Properties,
    pub transform: Transform2D,
}

pub(super) struct Text_Props {
    pub string: String,
    pub font: Font_Handle,
    pub font_size: u16,
    pub paint_props: Paint_Properties,
    pub screen_pos: Vec2f,
}

pub(super) fn add_texture_ws(
    batches: &mut Batches,
    texture: Texture_Handle,
    tex_rect: &Rect<i32>,
    color: Color,
    transform: &Transform2D,
) {
    batches
        .textures_ws
        .entry(texture)
        .or_insert_with(|| {
            (
                Vertex_Buffer::new(PrimitiveType::Quads, 65536, VertexBufferUsage::Stream),
                vec![],
            )
        })
        .1
        .push(Texture_Props {
            tex_rect: *tex_rect,
            color,
            transform: *transform,
        });
}

pub(super) fn add_rect_ws(
    batches: &mut Batches,
    rect: &Rect<f32>,
    props: &Paint_Properties,
    transform: &Transform2D,
) {
    batches.rects_ws.push(Rect_Props_Ws {
        rect: *rect,
        paint_props: *props,
        transform: *transform,
    });
}

pub(super) fn add_rect(batches: &mut Batches, rect: &Rect<f32>, props: &Paint_Properties) {
    batches.rects.push(Rect_Props {
        rect: *rect,
        paint_props: *props,
    });
}

pub(super) fn add_circle_ws(batches: &mut Batches, circle: &Circle, props: &Paint_Properties) {
    batches.circles_ws.push(Circle_Props_Ws {
        circle: *circle,
        paint_props: *props,
    });
}

pub(super) fn add_text_ws(
    batches: &mut Batches,
    text: super::Text_Props,
    props: &Paint_Properties,
    transform: &Transform2D,
) {
    let font = text.font();
    let font_size = text.font_size();
    batches.texts_ws.push(Text_Props_Ws {
        string: text.owned_string(),
        font,
        font_size,
        paint_props: *props,
        transform: *transform,
    });
}

pub(super) fn add_text(
    batches: &mut Batches,
    text: super::Text_Props,
    props: &Paint_Properties,
    screen_pos: Vec2f,
) {
    let font = text.font();
    let font_size = text.font_size();
    batches.texts.push(Text_Props {
        string: text.owned_string(),
        font,
        font_size,
        paint_props: *props,
        screen_pos,
    });
}

pub(super) fn add_line(batches: &mut Batches, from: render::Vertex, to: render::Vertex) {
    batches.lines.push(Line_Props { from, to });
}

pub fn clear_batches(batches: &mut Batches) {
    trace!("clear_batches");
    batches.textures_ws.clear();
    batches.rects_ws.clear();
    batches.rects.clear();
    batches.texts_ws.clear();
    batches.texts.clear();
    batches.lines.clear();
    batches.circles_ws.clear();
}

pub fn draw_batches(
    window: &mut Window_Handle,
    gres: &Gfx_Resources,
    batches: &mut Batches,
    camera: &Transform2D,
) {
    trace!("draw_all_batches");

    let inv_cam_transf = camera.inverse();
    //let win_size = Vec2f::from(Vec2u::from(crate::gfx::window::get_window_target_size(
    //window,
    //)));

    // @Speed: parallelize draw_rects_ws and draw_rects, probably together
    draw_rects_ws(window, gres, &inv_cam_transf, &batches.rects_ws);

    draw_textures_ws(window, gres, &inv_cam_transf, &mut batches.textures_ws);

    draw_circles_ws(window, gres, &inv_cam_transf, &batches.circles_ws);
    draw_rects(window, gres, &inv_cam_transf, &batches.rects);

    draw_texts_ws(window, gres, &inv_cam_transf, &batches.texts_ws);
    draw_texts(window, gres, &batches.texts);
}

fn draw_textures_ws(
    window: &mut Window_Handle,
    gres: &Gfx_Resources,
    inv_cam_transf: &Transform2D,
    textures_ws: &mut HashMap<Texture_Handle, (Vertex_Buffer, Vec<Texture_Props>)>,
) {
    for (tex_id, (vbuffer, tex_props)) in textures_ws {
        //let mut vbuf = render::start_draw_quads(tex_props.len());
        let texture = gres.get_texture(*tex_id);

        let n_texs = tex_props.len();
        let mut vertices = Vec::with_capacity(n_texs * 4);
        vertices.resize(n_texs * 4, sfml::graphics::Vertex::default());

        thread::scope(|s| {
            let n_threads = num_cpus::get();
            let mut vert_chunks = vertices.chunks_mut((n_texs / n_threads + 1) * 4);
            for chunk in tex_props.chunks(n_texs / n_threads + 1) {
                let vertices = vert_chunks.next().unwrap();
                s.spawn(move |_| {
                    for (i, tex_prop) in chunk.iter().enumerate() {
                        trace!("tex_batch");

                        let Texture_Props {
                            tex_rect,
                            color,
                            transform,
                        } = tex_prop;

                        let color = *color;
                        let uv: Rect<f32> = (*tex_rect).into();
                        let tex_size = Vec2f::new(tex_rect.width as _, tex_rect.height as _);
                        let render_transform = inv_cam_transf.combine(transform);

                        // Note: beware of the order of multiplications!
                        // Scaling the local positions must be done BEFORE multiplying the matrix!
                        let p1 = render_transform * (tex_size * v2!(-0.5, -0.5));
                        let p2 = render_transform * (tex_size * v2!(0.5, -0.5));
                        let p3 = render_transform * (tex_size * v2!(0.5, 0.5));
                        let p4 = render_transform * (tex_size * v2!(-0.5, 0.5));

                        // @Speed: investigate if culling here would benefit performance or not

                        let v1 = render::new_vertex(p1, color, v2!(uv.x, uv.y));
                        let v2 = render::new_vertex(p2, color, v2!(uv.x + uv.width, uv.y));
                        let v3 =
                            render::new_vertex(p3, color, v2!(uv.x + uv.width, uv.y + uv.height));
                        let v4 = render::new_vertex(p4, color, v2!(uv.x, uv.y + uv.height));

                        vertices[i * 4 + 0] = v1;
                        vertices[i * 4 + 1] = v2;
                        vertices[i * 4 + 2] = v3;
                        vertices[i * 4 + 3] = v4;
                        //render::add_quad(&mut vbuf, &v1, &v2, &v3, &v4);
                    }
                });
            }
        })
        .unwrap();

        {
            trace!("vbuf_update");
            vbuffer.update(&vertices, 0);
        }

        use sfml::graphics::RenderTarget;
        window.raw_handle_mut().draw_vertex_buffer(
            vbuffer,
            sfml::graphics::RenderStates {
                texture: Some(texture),
                ..sfml::graphics::RenderStates::default()
            },
        );
        //render::render_vbuf_texture(window, &vbuf, texture);
    }
}

fn draw_rects_ws(
    window: &mut Window_Handle,
    gres: &Gfx_Resources,
    inv_cam_transf: &Transform2D,
    rects_ws: &[Rect_Props_Ws],
) {
    let mut vbuf = render::start_draw_quads(rects_ws.len());
    for rect_props in rects_ws {
        trace!("rect_ws_batch");

        let Rect_Props_Ws {
            rect,
            paint_props,
            transform,
        } = rect_props;

        let color = paint_props.color;
        // @Incomplete: outline etc

        let render_transform = inv_cam_transf.combine(&transform);
        let rect_size = Vec2f::new(rect.width as _, rect.height as _);

        let p1 = render_transform * v2!(rect.x, rect.y);
        let p2 = render_transform * v2!(rect.x + rect.width, rect.y);
        let p3 = render_transform * v2!(rect.x + rect.width, rect.y + rect.height);
        let p4 = render_transform * v2!(rect.x, rect.y + rect.height);

        // @Speed: investigate if culling here would benefit performance or not

        let v1 = render::new_vertex(p1, color, Vec2f::default());
        let v2 = render::new_vertex(p2, color, Vec2f::default());
        let v3 = render::new_vertex(p3, color, Vec2f::default());
        let v4 = render::new_vertex(p4, color, Vec2f::default());
        render::add_quad(&mut vbuf, &v1, &v2, &v3, &v4);
    }
    render::render_vbuf(window, &vbuf, &Transform2D::default());
}

fn draw_rects(
    window: &mut Window_Handle,
    gres: &Gfx_Resources,
    inv_cam_transf: &Transform2D,
    rects: &[Rect_Props],
) {
    let mut vbuf = render::start_draw_quads(rects.len());
    for rect_props in rects {
        trace!("rect_batch");

        let Rect_Props { rect, paint_props } = rect_props;

        let color = paint_props.color;
        // @Incomplete: outline etc

        let p1 = v2!(rect.x, rect.y);
        let p2 = v2!(rect.x + rect.width, rect.y);
        let p3 = v2!(rect.x + rect.width, rect.y + rect.height);
        let p4 = v2!(rect.x, rect.y + rect.height);

        // @Speed: investigate if culling here would benefit performance or not

        let v1 = render::new_vertex(p1, color, Vec2f::default());
        let v2 = render::new_vertex(p2, color, Vec2f::default());
        let v3 = render::new_vertex(p3, color, Vec2f::default());
        let v4 = render::new_vertex(p4, color, Vec2f::default());
        render::add_quad(&mut vbuf, &v1, &v2, &v3, &v4);
    }
    render::render_vbuf(window, &vbuf, &Transform2D::default());
}

fn draw_texts_ws(
    window: &mut Window_Handle,
    gres: &Gfx_Resources,
    inv_cam_transf: &Transform2D,
    texts_ws: &[Text_Props_Ws],
) {
    for text_props in texts_ws {
        trace!("text_ws_batch");

        let Text_Props_Ws {
            string,
            font,
            font_size,
            paint_props,
            transform,
        } = text_props;
        let font = gres.get_font(*font);
        let mut text = render::backend::create_text(string, font, *font_size);

        let transform = inv_cam_transf.combine(&transform);
        render::backend::render_text_ws(
            window,
            &mut text,
            paint_props,
            &transform,
            &Transform2D::default(),
        );
    }
}

fn draw_texts(window: &mut Window_Handle, gres: &Gfx_Resources, texts: &[Text_Props]) {
    for text_props in texts {
        trace!("text_batch");

        let Text_Props {
            string,
            font,
            font_size,
            paint_props,
            screen_pos,
        } = text_props;
        let font = gres.get_font(*font);
        let mut text = render::backend::create_text(string, font, *font_size);

        render::backend::render_text(window, &mut text, paint_props, *screen_pos);
    }
}

fn draw_lines(window: &mut Window_Handle, gres: &Gfx_Resources, lines: &[Line_Props]) {
    let mut vbuf = render::start_draw_lines(lines.len());
    for line_props in lines {
        trace!("lines_batch");

        let Line_Props { from, to } = line_props;
        render::add_line(&mut vbuf, from, to);
    }
    render::render_vbuf(window, &vbuf, &Transform2D::default());
}

fn draw_circles_ws(
    window: &mut Window_Handle,
    gres: &Gfx_Resources,
    inv_cam_transf: &Transform2D,
    circles_ws: &[Circle_Props_Ws],
) {
    for circle_props in circles_ws {
        trace!("circle_ws_batch");

        let Circle_Props_Ws {
            circle,
            paint_props,
        } = circle_props;

        // @Incomplete: outline etc

        let transform = Transform2D::from_pos(circle.center);
        let render_transform = inv_cam_transf.combine(&transform);
        let circle = Circle {
            center: render_transform.position(),
            radius: render_transform.scale().x * circle.radius,
        };

        // @Speed: use a vertex buffer

        render::backend::fill_color_circle_ws(window, paint_props, circle, &Transform2D::default());
    }
}
