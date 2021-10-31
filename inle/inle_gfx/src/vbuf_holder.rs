use crate::render::{self, Vertex};
use crate::render::{Primitive_Type, Vertex_Buffer};
use crate::render_window::Render_Window_Handle;

pub struct Vertex_Buffer_Holder {
    pub vbuf: Vertex_Buffer,
    #[cfg(debug_assertions)]
    id: String,
}

impl Vertex_Buffer_Holder {
    pub fn with_initial_vertex_count(
        window: &mut Render_Window_Handle,
        initial_cap: u32,
        primitive: Primitive_Type,
        #[cfg(debug_assertions)] id: String,
    ) -> Self {
        Self {
            vbuf: render::new_vbuf(window, primitive, initial_cap),
            #[cfg(debug_assertions)]
            id,
        }
    }

    pub fn update(&mut self, vertices: &[Vertex], n_vertices: u32) {
        trace!("vbuf_holder::update");

        debug_assert!(vertices.len() <= std::u32::MAX as usize);

        debug_assert!(
            n_vertices <= render::vbuf_max_vertices(&self.vbuf),
            "Can't hold all the vertices! {} / {}",
            n_vertices,
            render::vbuf_max_vertices(&self.vbuf)
        );

        render::update_vbuf(&mut self.vbuf, &vertices[..n_vertices as _], 0);
        render::set_vbuf_cur_vertices(&mut self.vbuf, n_vertices);
    }

    pub fn grow(&mut self, window: &mut Render_Window_Handle, vertices_to_hold_at_least: u32) {
        trace!("vbuf_holder::grow");

        let new_cap = vertices_to_hold_at_least.next_power_of_two();
        ldebug!(
            "Growing Vertex_Buffer_Holder {} to hold {} vertices ({} requested).",
            self.id,
            new_cap,
            vertices_to_hold_at_least
        );

        let mut new_vbuf =
            render::new_vbuf(window, render::vbuf_primitive_type(&self.vbuf), new_cap);
        let _res = render::swap_vbuf(&mut new_vbuf, &mut self.vbuf);

        #[cfg(debug_assertions)]
        {
            assert!(_res, "Vertex Buffer copying failed ({})!", self.id);
        }

        render::dealloc_vbuf(&mut new_vbuf);
    }
}

impl Drop for Vertex_Buffer_Holder {
    fn drop(&mut self) {
        render::dealloc_vbuf(&mut self.vbuf);
    }
}
