use super::misc::check_gl_err;
use gl::types::*;
use std::ffi::c_void;
use std::{mem, ptr};

#[cfg(debug_assertions)]
use std::collections::HashSet;

const MIN_BUCKET_SIZE: usize = 32 * 1024;

pub struct Buffer_Allocators {
    pub array_buffer: Buffer_Allocator,

    /// This is deallocated every frame
    pub temp_array_buffer: Buffer_Allocator,
}

impl Default for Buffer_Allocators {
    fn default() -> Self {
        Self {
            array_buffer: Buffer_Allocator::new(gl::ARRAY_BUFFER),
            temp_array_buffer: Buffer_Allocator::new(gl::ARRAY_BUFFER),
        }
    }
}

impl Buffer_Allocators {
    pub fn destroy(&mut self) {
        self.array_buffer.destroy();
        self.temp_array_buffer.destroy();
    }

    #[cfg(debug_assertions)]
    /// Returns (permanent, temp)
    pub fn cur_allocated_buffer_handles(&self) -> (usize, usize) {
        (
            self.array_buffer.cur_allocated.len(),
            self.temp_array_buffer.cur_allocated.len(),
        )
    }
}

/// Buffer_Allocator holds a list of buckets, each backed by one openGL VAO + VBO.
/// The backing VBOs are mapped write-only through the buckets' `mapped_ptr` and
/// never unmapped.
/// "Virtual" buffers are allocated from these buckets and can write to their allocated
/// memory range via their Buffer_Handle.
pub struct Buffer_Allocator {
    buckets: Vec<Buffer_Allocator_Bucket>,
    buf_type: GLenum,

    #[cfg(debug_assertions)]
    cur_allocated: HashSet<Non_Empty_Buffer_Handle>,
}

#[derive(Debug)]
pub struct Buffer_Handle {
    inner: Buffer_Handle_Inner,
}

#[derive(Debug)]
enum Buffer_Handle_Inner {
    Empty,
    Non_Empty(Non_Empty_Buffer_Handle),
}

#[derive(Debug)]
#[cfg_attr(debug_assertions, derive(PartialEq, Eq, Hash, Clone))]
struct Non_Empty_Buffer_Handle {
    vao: GLuint,
    vbo: GLuint,

    bucket_idx: u16,
    slot: Bucket_Slot,

    data: *mut c_void,
}

impl Buffer_Handle {
    pub fn vao(&self) -> GLuint {
        if let Buffer_Handle_Inner::Non_Empty(h) = &self.inner {
            h.vao
        } else {
            0
        }
    }

    pub fn vbo(&self) -> GLuint {
        if let Buffer_Handle_Inner::Non_Empty(h) = &self.inner {
            h.vbo
        } else {
            0
        }
    }

    /// `offset` and `len` are in bytes.
    pub fn write(&mut self, offset: usize, len: usize, data: *const c_void) {
        if let Buffer_Handle_Inner::Non_Empty(h) = &mut self.inner {
            debug_assert!(h.slot.start + h.slot.len >= offset + len);
            unsafe {
                let write_start = h.data.add(offset / mem::size_of::<c_void>());
                ptr::copy_nonoverlapping(data, write_start, len / mem::size_of::<c_void>());
            }
        }
    }
}

impl Buffer_Allocator {
    pub fn new(buf_type: GLenum) -> Self {
        debug_assert!(buf_type == gl::ARRAY_BUFFER, "Currently we are not really supporting buffers other than ARRAY_BUFFER. If we need to, the VAO should be made optional");

        Self {
            buckets: vec![],
            buf_type,
            #[cfg(debug_assertions)]
            cur_allocated: HashSet::default(),
        }
    }

    pub fn destroy(&mut self) {
        let bucket_vbos = self
            .buckets
            .iter()
            .map(|bucket| bucket.vbo)
            .collect::<Vec<_>>();
        let bucket_vaos = self
            .buckets
            .iter()
            .map(|bucket| bucket.vao)
            .collect::<Vec<_>>();
        unsafe {
            gl::DeleteBuffers(bucket_vbos.len() as _, bucket_vbos.as_ptr() as _);
            gl::DeleteVertexArrays(bucket_vaos.len() as _, bucket_vaos.as_ptr() as _);
        }
    }

    pub fn dealloc_all(&mut self) {
        for bucket in &mut self.buckets {
            reset_bucket(bucket);
        }
        #[cfg(debug_assertions)]
        {
            self.cur_allocated.clear();
        }
    }

    pub fn allocate(&mut self, min_capacity: usize) -> Buffer_Handle {
        if min_capacity == 0 {
            return Buffer_Handle {
                inner: Buffer_Handle_Inner::Empty,
            };
        }

        let capacity_to_allocate = min_capacity.max(MIN_BUCKET_SIZE);

        let (bucket_idx, slot) = if let Some((bucket_idx, free_slot_idx)) =
            self.find_first_bucket_with_capacity(min_capacity)
        {
            let slot = allocate_from_bucket(
                &mut self.buckets[bucket_idx],
                free_slot_idx,
                capacity_to_allocate,
            );

            (bucket_idx, slot)
        } else {
            let mut bucket = allocate_bucket(self.buf_type, capacity_to_allocate);
            let slot = allocate_from_bucket(&mut bucket, 0, capacity_to_allocate);
            self.buckets.push(bucket);

            (self.buckets.len() - 1, slot)
        };

        let bucket = &self.buckets[bucket_idx];

        let data = unsafe { bucket.mapped_ptr.add(slot.start / mem::size_of::<c_void>()) };
        let h = Non_Empty_Buffer_Handle {
            bucket_idx: bucket_idx as _,
            slot,
            vao: bucket.vao,
            vbo: bucket.vbo,
            data,
        };

        #[cfg(debug_assertions)]
        {
            self.cur_allocated.insert(h.clone());
        }

        Buffer_Handle {
            inner: Buffer_Handle_Inner::Non_Empty(h),
        }
    }

    pub fn deallocate(&mut self, handle: Buffer_Handle) {
        if let Buffer_Handle_Inner::Non_Empty(h) = handle.inner {
            debug_assert!(self.cur_allocated.contains(&h));
            deallocate_in_bucket(&mut self.buckets[h.bucket_idx as usize], h.slot);

            #[cfg(debug_assertions)]
            {
                self.cur_allocated.remove(&h);
            }
        }
    }

    fn find_first_bucket_with_capacity(&self, capacity: usize) -> Option<(usize, usize)> {
        for (bucket_idx, bucket) in self.buckets.iter().enumerate() {
            if let Some(free_slot_idx) = bucket_has_contiguous_capacity(&bucket, capacity) {
                return Some((bucket_idx, free_slot_idx));
            }
        }
        None
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(debug_assertions, derive(Hash))]
struct Bucket_Slot {
    pub start: usize,
    pub len: usize,
}

impl Bucket_Slot {
    pub fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }

    pub fn contains(&self, other: &Bucket_Slot) -> bool {
        self.start <= other.start && self.start + self.len >= other.start + other.len
    }
}

impl std::cmp::PartialOrd for Bucket_Slot {
    fn partial_cmp(&self, other: &Bucket_Slot) -> Option<std::cmp::Ordering> {
        self.start.partial_cmp(&other.start)
    }
}

struct Buffer_Allocator_Bucket {
    vao: GLuint,
    vbo: GLuint,

    free_list: Vec<Bucket_Slot>,
    capacity: usize,

    mapped_ptr: *mut c_void,
}

fn allocate_bucket(buf_type: GLenum, capacity: usize) -> Buffer_Allocator_Bucket {
    let (mut vao, mut vbo) = (0, 0);
    let mapped_ptr;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        debug_assert!(vao != 0);

        gl::GenBuffers(1, &mut vbo);
        debug_assert!(vbo != 0);

        gl::BindVertexArray(vao);

        gl::BindBuffer(buf_type, vbo);

        gl::BufferStorage(
            buf_type,
            capacity as _,
            ptr::null(),
            gl::DYNAMIC_STORAGE_BIT
                | gl::MAP_WRITE_BIT
                | gl::MAP_PERSISTENT_BIT
                | gl::MAP_COHERENT_BIT,
        );

        mapped_ptr = gl::MapNamedBuffer(vbo, gl::WRITE_ONLY);

        check_gl_err();
    }

    ldebug!(
        "Buffer_Allocator: allocated new bucket with capacity {} B",
        capacity
    );

    Buffer_Allocator_Bucket {
        vao,
        vbo,
        free_list: vec![Bucket_Slot {
            start: 0,
            len: capacity,
        }],
        capacity,
        mapped_ptr,
    }
}

// Returns Some(free_slot_idx) or None
fn bucket_has_contiguous_capacity(bucket: &Buffer_Allocator_Bucket, cap: usize) -> Option<usize> {
    bucket.free_list.iter().position(|slot| slot.len >= cap)
}

fn is_bucket_slot_free(bucket: &Buffer_Allocator_Bucket, slot: &Bucket_Slot) -> bool {
    bucket.free_list.iter().any(|s| s.contains(slot))
}

fn allocate_from_bucket(
    bucket: &mut Buffer_Allocator_Bucket,
    free_slot_idx: usize,
    len: usize,
) -> Bucket_Slot {
    let slot = Bucket_Slot::new(bucket.free_list[free_slot_idx].start, len);

    debug_assert!(len <= bucket.free_list[free_slot_idx].len);
    debug_assert!(is_bucket_slot_free(bucket, &slot));

    bucket.free_list[free_slot_idx].len -= len;
    let slot_new_len = bucket.free_list[free_slot_idx].len;
    if slot_new_len == 0 {
        bucket.free_list.remove(free_slot_idx);
    } else {
        let new_off = bucket.free_list[free_slot_idx].start + len;
        bucket
            .free_list
            .insert(free_slot_idx + 1, Bucket_Slot::new(new_off, slot_new_len));
    }

    debug_assert!(free_list_is_sorted(&bucket.free_list));
    debug_assert!(
        bucket.free_list.is_empty()
            || (bucket.free_list[bucket.free_list.len() - 1].start
                + bucket.free_list[bucket.free_list.len() - 1].len
                <= bucket.capacity)
    );

    slot
}

fn free_list_is_sorted(list: &[Bucket_Slot]) -> bool {
    for i in 1..list.len() {
        if list[i - 1].start + list[i - 1].len > list[i].start {
            return false;
        }
    }
    true
}

fn deallocate_in_bucket(bucket: &mut Buffer_Allocator_Bucket, slot: Bucket_Slot) {
    debug_assert!(!is_bucket_slot_free(bucket, &slot));

    if let Some(insert_pos) = bucket.free_list.iter().position(|s| s.start > slot.start) {
        if insert_pos > 0
            && bucket.free_list[insert_pos - 1].start + bucket.free_list[insert_pos - 1].len
                == slot.start
        {
            bucket.free_list[insert_pos - 1].len += slot.len;
        } else {
            bucket.free_list.insert(insert_pos, slot);
        }
    } else {
        let last_idx = bucket.free_list.len() - 1;
        if !bucket.free_list.is_empty()
            && bucket.free_list[last_idx].start + bucket.free_list[last_idx].len == slot.start
        {
            bucket.free_list[last_idx].len += slot.len;
        } else {
            bucket.free_list.push(slot);
        }
    }
}

fn reset_bucket(bucket: &mut Buffer_Allocator_Bucket) {
    bucket.free_list = vec![Bucket_Slot::new(0, bucket.capacity)];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore] // until we can load gl ptrs in tests
    #[test]
    fn test_allocate_from_bucket() {
        let mut bucket = allocate_bucket(gl::ARRAY_BUFFER, 100);
        debug_assert!(is_bucket_slot_free(&bucket, &Bucket_Slot::new(0, 100)));
    }
}
