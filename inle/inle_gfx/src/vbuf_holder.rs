use crate::render::{Vertex_Buffer, Primitive_Type};
use crate::render::{self, Vertex};
use inle_common::colors;

// @Cutnpaste from batcher.rs
fn null_vertex() -> Vertex {
    render::new_vertex(v2!(0., 0.), colors::TRANSPARENT, v2!(0., 0.))
}

pub struct Vertex_Buffer_Holder {
    pub vbuf: Vertex_Buffer,
    #[cfg(debug_assertions)]
    id: String,
}

impl Vertex_Buffer_Holder {
    pub fn with_initial_vertex_count(
        initial_cap: u32,
        primitive: Primitive_Type,
        #[cfg(debug_assertions)] id: String,
    ) -> Self {
        Self {
            vbuf: render::new_vbuf(primitive, initial_cap),
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

        render::update_vbuf(&mut self.vbuf, vertices, 0);
        render::set_vbuf_cur_vertices(&mut self.vbuf, vertices.len() as u32);
    }

    pub fn grow(&mut self, vertices_to_hold_at_least: u32) {
        let new_cap = vertices_to_hold_at_least.next_power_of_two();
        ldebug!(
            "Growing Vertex_Buffer_Holder {} to hold {} vertices ({} requested).",
            self.id,
            new_cap,
            vertices_to_hold_at_least
        );

        let mut new_vbuf = render::new_vbuf(render::vbuf_primitive_type(&self.vbuf), new_cap);
        let _res = render::swap_vbuf(&mut new_vbuf, &mut self.vbuf);
        #[cfg(debug_assertions)]
        {
            debug_assert!(_res, "Vertex Buffer copying failed ({})!", self.id);
        }
    }
}
