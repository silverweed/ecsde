use crate::common::colors::Color;
use crate::common::rect::Rect;
use crate::common::transform::Transform2D;
use crate::gfx::paint_props::Paint_Properties;
use crate::gfx::window::Window_Handle;
use crate::resources::gfx::{Gfx_Resources, Texture_Handle};
use crate::common::vector::Vec2f;
use std::collections::HashMap;

#[derive(Default)]
pub struct Batches {
    textures_ws: HashMap<Texture_Handle, Vec<Texture_Props>>,
    rects_ws: Vec<Rect_Props>,
}

pub(super) struct Texture_Props {
    pub tex_rect: Rect<i32>,
    pub color: Color,
    pub transform: Transform2D,
}

pub(super) struct Rect_Props {
    pub rect: Rect<f32>,
    pub paint_props: Paint_Properties,
    pub transform: Transform2D,
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
    batches.rects_ws.push(Rect_Props {
        rect: *rect,
        paint_props: *props,
        transform: *transform,
    });
}

pub fn draw_all_batches(
    window: &mut Window_Handle,
    gres: &Gfx_Resources,
    batches: &Batches,
    camera: &Transform2D,
) {
    use crate::gfx::render::{start_draw_quads, add_quad, new_vertex, render_vbuf_texture};

    let inv_cam_transf = camera.inverse(); //camera.get_matrix_sfml().inverse();
    for (tex_id, tex_props) in &batches.textures_ws {
        let mut vbuf = start_draw_quads(tex_props.len());
        let texture = gres.get_texture(*tex_id);

        for tex_prop in tex_props {
            let Texture_Props {
                tex_rect,
                color,
                transform,
            } = tex_prop;

            let mut render_transform = inv_cam_transf;
            render_transform = render_transform.combine(transform);
            //render_transform.combine(&transform.get_matrix_sfml());
            //
            println!("result:");
            crate::common::transform::matrix_pretty_print(&render_transform.get_matrix());

            let color= *color;
            let uv: Rect<f32> = (*tex_rect).into();
            let tex_size = Vec2f::new(tex_rect.width as _, tex_rect.height as _);
            let p = tex_size * Vec2f::new(-0.5, -0.5);
            println!("p = {:?}, t*p = {:?}", p, render_transform * p);
            let v1 = new_vertex(
                render_transform * tex_size * Vec2f::new(-0.5, -0.5),
                //tex_size * Vec2f::new(-0.5, -0.5),
                color,
                Vec2f::new(uv.x, uv.y),
            );
            let v2 = new_vertex(
                render_transform * tex_size * Vec2f::new(0.5, -0.5),
                //tex_size * Vec2f::new(0.5, -0.5),
                color,
                Vec2f::new(uv.x + uv.width, uv.y),
            );
            let v3 = new_vertex(
                render_transform * tex_size * Vec2f::new(0.5, 0.5),
                //tex_size * Vec2f::new(0.5, 0.5),
                color,
                Vec2f::new(uv.x + uv.width, uv.y + uv.height),
            );
            let v4 = new_vertex(
                render_transform * tex_size * Vec2f::new(-0.5, 0.5),
                //tex_size * Vec2f::new(-0.5, 0.5),
                color,
                Vec2f::new(uv.x, uv.y + uv.height),
            );
            println!("adding quad\n\t{:?}\n\t{:?}\n\t{:?}\n\t{:?}",
                     v1.position,
                     v2.position,
                     v3.position,
                     v4.position,
                     );
            add_quad(&mut vbuf, &v1, &v2, &v3, &v4);

        }

        render_vbuf_texture(window, &vbuf, texture);
    }
}
