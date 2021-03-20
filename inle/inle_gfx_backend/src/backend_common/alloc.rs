use super::misc::check_gl_err;
use gl::types::*;
use std::ffi::c_void;
use std::ptr;

#[cfg(debug_assertions)]
use std::collections::HashSet;

const MIN_BUCKET_SIZE: usize = 32 * 1024;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Hash))]
pub enum Buffer_Allocator_Id {
    Array_Permanent = 0,
    Array_Temporary = 1,
}

pub struct Buffer_Allocators {
    buffers: [Buffer_Allocator; 2],
}

impl Default for Buffer_Allocators {
    fn default() -> Self {
        Self {
            buffers: [
                Buffer_Allocator::new(gl::ARRAY_BUFFER, Buffer_Allocator_Id::Array_Permanent),
                Buffer_Allocator::new(gl::ARRAY_BUFFER, Buffer_Allocator_Id::Array_Temporary),
            ],
        }
    }
}

impl Buffer_Allocators {
    pub fn destroy(&mut self) {
        for buf in &mut self.buffers {
            buf.destroy();
        }
    }

    pub fn dealloc_all_temp(&mut self) {
        self.buffers[Buffer_Allocator_Id::Array_Temporary as usize].dealloc_all();
    }

    pub fn get_buffer_mut(&mut self, id: Buffer_Allocator_Id) -> &mut Buffer_Allocator {
        &mut self.buffers[id as usize]
    }
}

/// Buffer_Allocator holds a list of buckets, each backed by one openGL VAO + VBO.
/// "Virtual" buffers are allocated from these buckets and can write to their allocated
/// memory range via their Buffer_Handle.
pub struct Buffer_Allocator {
    buckets: Vec<Buffer_Allocator_Bucket>,
    buf_type: GLenum,

    id: Buffer_Allocator_Id,

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
    allocator_id: Buffer_Allocator_Id,
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

    pub fn allocator_id(&self) -> Buffer_Allocator_Id {
        if let Buffer_Handle_Inner::Non_Empty(h) = &self.inner {
            h.allocator_id
        } else {
            // @Robustness: this is quite sloppy and relies on other parts of the code doing the proper checks;
            // maybe change logic a bit so we never end up here
            Buffer_Allocator_Id::Array_Temporary
        }
    }
}

impl Buffer_Allocator {
    pub fn new(buf_type: GLenum, id: Buffer_Allocator_Id) -> Self {
        debug_assert!(buf_type == gl::ARRAY_BUFFER, "Currently we are not really supporting buffers other than ARRAY_BUFFER. If we need to, the VAO should be made optional");

        Self {
            buckets: vec![],
            buf_type,
            id,
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

        let h = Non_Empty_Buffer_Handle {
            bucket_idx: bucket_idx as _,
            slot,
            vao: bucket.vao,
            vbo: bucket.vbo,
            allocator_id: self.id,
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
            #[cfg(debug_assertions)]
            {
                debug_assert!(self.cur_allocated.contains(&h));
            }

            deallocate_in_bucket(&mut self.buckets[h.bucket_idx as usize], h.slot);

            #[cfg(debug_assertions)]
            {
                self.cur_allocated.remove(&h);
            }
        }
    }

    pub fn update_buffer(
        &mut self,
        handle: &Buffer_Handle,
        offset: usize,
        len: usize,
        data: *const c_void,
    ) {
        if let Buffer_Handle_Inner::Non_Empty(h) = &handle.inner {
            #[cfg(debug_assertions)]
            {
                debug_assert!(self.cur_allocated.contains(&h));
            }
            write_to_bucket(
                &mut self.buckets[h.bucket_idx as usize],
                h,
                offset,
                len,
                data,
            );
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
    buf_type: GLenum,

    free_list: Vec<Bucket_Slot>,
    capacity: usize,
}

fn allocate_bucket(buf_type: GLenum, capacity: usize) -> Buffer_Allocator_Bucket {
    let (mut vao, mut vbo) = (0, 0);
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
            gl::DYNAMIC_STORAGE_BIT,
        );

        check_gl_err();
    }

    lverbose!(
        "Buffer_Allocator: allocated new bucket with capacity {} B",
        capacity
    );

    Buffer_Allocator_Bucket {
        vao,
        vbo,
        buf_type,
        free_list: vec![Bucket_Slot {
            start: 0,
            len: capacity,
        }],
        capacity,
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

fn write_to_bucket(
    bucket: &mut Buffer_Allocator_Bucket,
    handle: &Non_Empty_Buffer_Handle,
    offset: usize,
    len: usize,
    data: *const c_void,
) {
    debug_assert!(!is_bucket_slot_free(bucket, &handle.slot));
    debug_assert!(len <= handle.slot.len);

    unsafe {
        gl::BindBuffer(bucket.buf_type, bucket.vbo);
        gl::BufferSubData(
            bucket.buf_type,
            (handle.slot.start + offset) as _,
            len as _,
            data,
        );
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
