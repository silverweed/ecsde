use crate::painter::Debug_Painter;
use inle_common::colors;
use inle_common::paint_props::Paint_Properties;
use inle_gfx_backend::backend_common::alloc::*;
use inle_math::transform::Transform2D;

const WIDTH: f32 = 400.; // @Robustness: we should refer to the window size
const HEIGHT: f32 = 15.;

pub fn debug_draw_buffer_allocators(allocators: &Buffer_Allocators, painter: &mut Debug_Painter) {
    let mut free_paint_props = Paint_Properties {
        color: colors::rgb(180, 60, 0),
        border_color: colors::rgb(20, 160, 50),
        border_thick: 2.,
        ..Default::default()
    };
    let mut occupied_paint_props = Paint_Properties {
        color: colors::rgb(20, 180, 20),
        border_thick: 1.,
        ..free_paint_props
    };

    let temp_alloc = allocators.get_alloc_mut(
        inle_gfx_backend::backend_common::alloc::Buffer_Allocator_Id::Array_Temporary,
    );
    debug_draw_buffer_allocator(
        &temp_alloc.borrow(),
        painter,
        0.,
        free_paint_props,
        occupied_paint_props,
    );

    free_paint_props.color = colors::rgb(0, 180, 100);
    occupied_paint_props.color = colors::rgb(150, 80, 0);

    debug_draw_buffer_allocator(
        &allocators
            .get_alloc_mut(
                inle_gfx_backend::backend_common::alloc::Buffer_Allocator_Id::Array_Permanent,
            )
            .borrow(),
        painter,
        (temp_alloc.borrow().get_buckets().len() + 2) as f32 * HEIGHT,
        free_paint_props,
        occupied_paint_props,
    );
}

fn debug_draw_buffer_allocator(
    alloc: &Buffer_Allocator,
    painter: &mut Debug_Painter,
    height_offset: f32,
    free_paint_props: Paint_Properties,
    occupied_paint_props: Paint_Properties,
) {
    for (i, bucket) in alloc.get_buckets().iter().enumerate() {
        debug_draw_bucket(
            bucket,
            painter,
            i,
            height_offset,
            free_paint_props,
            occupied_paint_props,
        );
    }
}

fn debug_draw_bucket(
    bucket: &Buffer_Allocator_Bucket,
    painter: &mut Debug_Painter,
    idx: usize,
    base_height_offset: f32,
    free_paint_props: Paint_Properties,
    occupied_paint_props: Paint_Properties,
) {
    let pos = v2!(
        10. + (idx / 45) as f32 * WIDTH,
        (idx % 45) as f32 * HEIGHT + base_height_offset
    ) - v2!(700., 400.); // @Temporary!

    painter.add_rect(
        v2!(WIDTH, HEIGHT),
        &Transform2D::from_pos(pos),
        occupied_paint_props,
    );

    for (i, slot) in bucket.free_list.iter().enumerate() {
        let slot_size = WIDTH * slot.len as f32 / bucket.capacity as f32;
        let slot_start = WIDTH * slot.start as f32 / bucket.capacity as f32;
        let mpos = pos + v2!(slot_start, 0.);
        painter.add_rect(
            v2!(slot_size, HEIGHT),
            &Transform2D::from_pos(mpos),
            free_paint_props,
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
