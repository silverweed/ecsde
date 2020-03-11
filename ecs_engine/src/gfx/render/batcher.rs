use crate::common::colors::Color;
use crate::common::rect::Rect;
use crate::common::transform::Transform2D;
use crate::common::vector::Vec2f;
use crate::gfx::paint_props::Paint_Properties;
use crate::gfx::window::Window_Handle;
use crate::resources::gfx::{Gfx_Resources, Texture_Handle};
use std::collections::HashMap;

#[derive(Default)]
pub struct Batches {
    textures_ws: HashMap<Texture_Handle, Vec<Texture_Props>>,
    rects_ws: Vec<Rect_Props_Ws>,
    rects: Vec<Rect_Props>,
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
        .or_insert_with(|| vec![])
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

pub fn clear_batches(batches: &mut Batches) {
    trace!("clear_batches");
    batches.textures_ws.clear();
    batches.rects_ws.clear();
    batches.rects.clear();
}

pub fn draw_batches(
    window: &mut Window_Handle,
    gres: &Gfx_Resources,
    batches: &Batches,
    camera: &Transform2D,
) {
    trace!("draw_all_batches");

    use crate::gfx::render::{
        add_quad, new_vertex, render_vbuf, render_vbuf_texture, start_draw_quads,
    };

    let inv_cam_transf = camera.inverse();
    for (tex_id, tex_props) in &batches.textures_ws {
        let mut vbuf = start_draw_quads(tex_props.len());
        let texture = gres.get_texture(*tex_id);

        for tex_prop in tex_props {
            trace!("tex_batch");

            let Texture_Props {
                tex_rect,
                color,
                transform,
            } = tex_prop;

            let mut render_transform = inv_cam_transf;
            render_transform = render_transform.combine(transform);

            let color = *color;
            let uv: Rect<f32> = (*tex_rect).into();
            let tex_size = Vec2f::new(tex_rect.width as _, tex_rect.height as _);

            // Note: beware of the order of multiplications!
            // Scaling the local positions must be done BEFORE multiplying the matrix!
            let p1 = render_transform * (tex_size * v2!(-0.5, -0.5));
            let p2 = render_transform * (tex_size * v2!(0.5, -0.5));
            let p3 = render_transform * (tex_size * v2!(0.5, 0.5));
            let p4 = render_transform * (tex_size * v2!(-0.5, 0.5));

            let v1 = new_vertex(p1, color, v2!(uv.x, uv.y));
            let v2 = new_vertex(p2, color, v2!(uv.x + uv.width, uv.y));
            let v3 = new_vertex(p3, color, v2!(uv.x + uv.width, uv.y + uv.height));
            let v4 = new_vertex(p4, color, v2!(uv.x, uv.y + uv.height));
            add_quad(&mut vbuf, &v1, &v2, &v3, &v4);
        }

        render_vbuf_texture(window, &vbuf, texture);
    }

    {
        let mut vbuf = start_draw_quads(batches.rects_ws.len());
        for rect_props in &batches.rects_ws {
            trace!("rect_ws_batch");

            let Rect_Props_Ws {
                rect,
                paint_props,
                transform,
            } = rect_props;

            let mut render_transform = inv_cam_transf;
            render_transform = render_transform.combine(&transform);

            let color = paint_props.color;
            // @Incomplete: outline etc

            let rect_size = Vec2f::new(rect.width as _, rect.height as _);

            // Note: beware of the order of multiplications!
            // Scaling the local positions must be done BEFORE multiplying the matrix!
            let p1 = render_transform * (rect_size * v2!(-0.5, -0.5));
            let p2 = render_transform * (rect_size * v2!(0.5, -0.5));
            let p3 = render_transform * (rect_size * v2!(0.5, 0.5));
            let p4 = render_transform * (rect_size * v2!(-0.5, 0.5));

            let v1 = new_vertex(p1, color, Vec2f::default());
            let v2 = new_vertex(p2, color, Vec2f::default());
            let v3 = new_vertex(p3, color, Vec2f::default());
            let v4 = new_vertex(p4, color, Vec2f::default());
            add_quad(&mut vbuf, &v1, &v2, &v3, &v4);
        }
        render_vbuf(window, &vbuf, &Transform2D::default());
    }

    {
        let mut vbuf = start_draw_quads(batches.rects.len());
        for rect_props in &batches.rects {
            trace!("rect_batch");

            let Rect_Props { rect, paint_props } = rect_props;

            let color = paint_props.color;
            // @Incomplete: outline etc

            let p1 = v2!(rect.x, rect.y);
            let p2 = v2!(rect.x + rect.width, rect.y);
            let p3 = v2!(rect.x + rect.width, rect.y + rect.height);
            let p4 = v2!(rect.x, rect.y + rect.height);

            let v1 = new_vertex(p1, color, Vec2f::default());
            let v2 = new_vertex(p2, color, Vec2f::default());
            let v3 = new_vertex(p3, color, Vec2f::default());
            let v4 = new_vertex(p4, color, Vec2f::default());
            add_quad(&mut vbuf, &v1, &v2, &v3, &v4);
        }
        render_vbuf(window, &vbuf, &Transform2D::default());
    }
}
