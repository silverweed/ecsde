use crate::painter::Debug_Painter;
use inle_common::colors;
use inle_common::paint_props::Paint_Properties;
use inle_gfx_backend::backend_common::alloc::*;
use inle_math::rect::Rectf;
use inle_math::transform::Transform2D;

pub fn debug_draw_buffer_allocator(alloc: &Buffer_Allocator, painter: &mut Debug_Painter) {
    for (i, bucket) in alloc.get_buckets().iter().enumerate() {
        debug_draw_bucket(bucket, painter, i);
    }
}

fn debug_draw_bucket(bucket: &Buffer_Allocator_Bucket, painter: &mut Debug_Painter, idx: usize) {
    let paint_props = Paint_Properties {
        color: colors::rgb(180, 60, 0),
        border_color: colors::rgb(20, 160, 50),
        border_thick: 2.,
        ..Default::default()
    };

    const WIDTH: f32 = 400.; // @Robustness: we should refer to the window size
    const HEIGHT: f32 = 15.;

    let pos = v2!(10. + (idx / 45) as f32 * WIDTH, (idx % 45) as f32 * HEIGHT) - v2!(700., 400.); // @Temporary!

    painter.add_rect(v2!(WIDTH, HEIGHT), &Transform2D::from_pos(pos), paint_props);

    for (i, slot) in bucket.free_list.iter().enumerate() {
        let slot_size = WIDTH * slot.len as f32 / bucket.capacity as f32;
        let slot_start = WIDTH * slot.start as f32 / bucket.capacity as f32;
        let paint_props = Paint_Properties {
            color: colors::rgb(20, 160, 0),
            border_thick: 1.,
            ..paint_props
        };
        let mpos = pos + v2!(slot_start, 0.);
        painter.add_rect(
            v2!(slot_size, HEIGHT),
            &Transform2D::from_pos(mpos),
            paint_props,
        );
        painter.add_text(
            &format!(
                "{}: {}",
                i,
                inle_common::units::format_bytes_pretty(slot.len)
            ),
            mpos,
            10,
            colors::BLUE,
        );
    }
}
