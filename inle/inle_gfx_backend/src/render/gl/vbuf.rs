use crate::render::Primitive_Type;
use crate::backend_common::alloc::{
    Buffer_Allocator_Id, Buffer_Allocator_Ptr, Buffer_Handle, EMPTY_BUFFER_HANDLE,
};
use std::cell::Cell;
use std::mem;

type Vertex = crate::backend_common::types::Vertex;

/// A Vertex_Buffer is an unresizable vertex buffer that accepts vertices in this format:
/// (location = 0) vec4 color;
/// (location = 1) vec2 pos;
/// (location = 2) vec2 tex_coords;
pub struct Vertex_Buffer {
    pub(super) max_vertices: u32,
    pub(super) primitive_type: Primitive_Type,
    pub(super) buf: Buffer_Handle,
    pub(super) parent_alloc: Buffer_Allocator_Ptr,
    pub(super) vertices: Vec<Vertex>,
    pub(super) needs_transfer_to_gpu: Cell<bool>,

   #[cfg(debug_assertions)]
    pub(super) buf_alloc_id: Buffer_Allocator_Id,
}

impl Vertex_Buffer {
    pub(super) fn new(
        buf_allocator_ptr: Buffer_Allocator_Ptr,
        primitive_type: Primitive_Type,
        max_vertices: u32,
        #[cfg(debug_assertions)]
        buf_alloc_id: Buffer_Allocator_Id,
    ) -> Self {
        let mut buffer_allocator = buf_allocator_ptr.borrow_mut();
        let buf = buffer_allocator.allocate(max_vertices as usize * mem::size_of::<Vertex>());

        if max_vertices == 0 {
            lwarn!("Creating a Vertex_Buffer with max_vertices = 0");
        }

        Self {
            buf,
            max_vertices,
            primitive_type,
            parent_alloc: buf_allocator_ptr.clone(),
            vertices: Vec::with_capacity(max_vertices as usize),
            needs_transfer_to_gpu: Cell::new(false),
            #[cfg(debug_assertions)]
            buf_alloc_id,
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for Vertex_Buffer {
    fn drop(&mut self) {
        assert!(self.buf == EMPTY_BUFFER_HANDLE || self.buf_alloc_id == Buffer_Allocator_Id::Array_Temporary,
            "A manual lifetime vertex buffer was not deallocated before being dropped!");
    }
}

#[track_caller]
#[inline(always)]
pub fn check_vbuf_valid(vbuf: &Vertex_Buffer) {
    debug_assert!(vbuf.buf != EMPTY_BUFFER_HANDLE, "Vertex Buffer was invalid!");
}

