use super::misc::check_gl_err;
use gl::types::*;
use std::ptr;

const MIN_BUCKET_SIZE: usize = 32 * 1024;

pub struct Buffer_Allocators {
    pub array_buffer: Buffer_Allocator,
}

impl Default for Buffer_Allocators {
    fn default() -> Self {
        Self {
            array_buffer: Buffer_Allocator::new(gl::ARRAY_BUFFER),
        }
    }
}

pub struct Buffer_Allocator {
    buckets: Vec<Buffer_Allocator_Bucket>,
    buf_type: GLenum,
}

pub struct Buffer_Handle {
    bucket_idx: u16,
    slot: Bucket_Slot,
}

impl Buffer_Allocator {
    pub fn new(buf_type: GLenum) -> Self {
        Self {
            buckets: vec![],
            buf_type,
        }
    }

    pub fn dealloc_all(&mut self) {
        let bucket_idx = self
            .buckets
            .iter()
            .map(|bucket| bucket.id)
            .collect::<Vec<_>>();
        unsafe {
            gl::DeleteBuffers(bucket_idx.len() as _, bucket_idx.as_ptr() as _);
        }
    }

    pub fn allocate(&mut self, min_capacity: usize) -> Buffer_Handle {
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

        Buffer_Handle {
            bucket_idx: bucket_idx as _,
            slot,
        }
    }

    pub fn deallocate(&mut self, handle: Buffer_Handle) {
        deallocate_in_bucket(&mut self.buckets[handle.bucket_idx as usize], handle.slot);
    }

    pub fn update_buffer(
        &mut self,
        handle: &Buffer_Handle,
        offset: usize,
        length: usize,
        data: *const std::ffi::c_void,
    ) {
        write_data_to_bucket(
            &mut self.buckets[handle.bucket_idx as usize],
            &handle.slot,
            offset,
            length,
            data,
        );
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

#[derive(Copy, Clone, PartialEq)]
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
    id: GLuint,

    free_list: Vec<Bucket_Slot>,
}

fn allocate_bucket(buf_type: GLenum, capacity: usize) -> Buffer_Allocator_Bucket {
    let mut id = 0;
    unsafe {
        gl::GenBuffers(1, &mut id);
        gl::BindBuffer(buf_type, id);

        gl::BufferStorage(
            buf_type,
            capacity as _,
            ptr::null(),
            gl::DYNAMIC_STORAGE_BIT,
        );
    }

    Buffer_Allocator_Bucket {
        id,
        free_list: vec![Bucket_Slot {
            start: 0,
            len: capacity,
        }],
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

    slot
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
        if bucket.free_list.len() > 0
            && bucket.free_list[last_idx].start + bucket.free_list[last_idx].len == slot.start
        {
            bucket.free_list[last_idx].len += slot.len;
        } else {
            bucket.free_list.push(slot);
        }
    }
}

fn write_data_to_bucket(
    bucket: &mut Buffer_Allocator_Bucket,
    slot: &Bucket_Slot,
    offset_in_slot: usize,
    length: usize,
    data: *const std::ffi::c_void,
) {
    debug_assert!(!is_bucket_slot_free(bucket, slot));
    debug_assert!(length <= slot.len - offset_in_slot);

    unsafe {
        gl::NamedBufferSubData(
            bucket.id,
            (slot.start + offset_in_slot) as _,
            length as _,
            data,
        );
        check_gl_err();
    }
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
