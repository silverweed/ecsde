use super::misc::check_gl_err;
use gl::types::*;
use inle_common::units::{self, format_bytes_pretty};
use std::cell::RefCell;
use std::ffi::c_void;
use std::ptr;
use std::rc::Rc;

#[cfg(debug_assertions)]
use std::collections::HashSet;

const MIN_BUCKET_SIZE: usize = units::kilobytes(32);

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Hash))]
pub enum Buffer_Allocator_Id {
    Array_Permanent = 0,
    Array_Temporary = 1,
}

pub struct Buffer_Allocators {
    allocs: [Buffer_Allocator_Ptr; 2],
}

impl Default for Buffer_Allocators {
    fn default() -> Self {
        Self {
            allocs: [
                Rc::new(RefCell::new(Buffer_Allocator::new(
                    gl::ARRAY_BUFFER,
                    Buffer_Allocator_Id::Array_Permanent,
                ))),
                Rc::new(RefCell::new(Buffer_Allocator::new(
                    gl::ARRAY_BUFFER,
                    Buffer_Allocator_Id::Array_Temporary,
                ))),
            ],
        }
    }
}

impl Buffer_Allocators {
    pub fn destroy(&mut self) {
        for buf in &mut self.allocs {
            buf.borrow_mut().destroy();
        }
    }

    pub fn dealloc_all_temp(&mut self) {
        self.allocs[Buffer_Allocator_Id::Array_Temporary as usize]
            .borrow_mut()
            .dealloc_all();
    }

    pub fn get_alloc_mut(&self, id: Buffer_Allocator_Id) -> Buffer_Allocator_Ptr {
        self.allocs[id as usize].clone()
    }
}

/// Buffer_Allocator holds a list of buckets, each backed by one openGL VAO + VBO.
/// "Virtual" allocs are allocated from these buckets and can write to their allocated
/// memory range via their Buffer_Handle.
pub struct Buffer_Allocator {
    buckets: Vec<Buffer_Allocator_Bucket>,
    buf_type: GLenum,

    id: Buffer_Allocator_Id,

    #[cfg(debug_assertions)]
    cur_allocated: HashSet<Non_Empty_Buffer_Handle>,

    #[cfg(debug_assertions)]
    cur_allocated_bytes: usize,

    #[cfg(debug_assertions)]
    high_water_mark: usize,

    #[cfg(debug_assertions)]
    is_destroyed: bool,
}

// Needed for the buf_alloc_debug
#[cfg(debug_assertions)]
impl Buffer_Allocator {
    pub fn get_buckets(&self) -> &[Buffer_Allocator_Bucket] {
        &self.buckets
    }

    pub fn get_cur_allocated(&self) -> &HashSet<Non_Empty_Buffer_Handle> {
        &self.cur_allocated
    }
}

// @Speed @Robustness: using Rc allows us to store references to the parent buf allocators
// inside the vertex buffers, which is convenient since it avoids us passing the window every
// time we want to update the vbuf.
// However this is only works single-thread and has a tiny performance penalty, so eventually
// we may want to have a specialized structure that allows exclusive fast access to a portion
// of the buffer only to the vbuf that allocated it.
pub type Buffer_Allocator_Ptr = Rc<RefCell<Buffer_Allocator>>;

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
#[cfg_attr(debug_assertions, derive(PartialEq, Eq, Clone))]
pub struct Non_Empty_Buffer_Handle {
    vao: GLuint,
    vbo: GLuint,

    pub bucket_idx: u16,
    pub slot: Bucket_Slot,
    allocator_id: Buffer_Allocator_Id,

    #[cfg(debug_assertions)]
    writes: RefCell<Vec<Bucket_Slot>>,
}

#[cfg(debug_assertions)]
impl std::hash::Hash for Non_Empty_Buffer_Handle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vao.hash(state);
        self.vbo.hash(state);
        self.bucket_idx.hash(state);
        self.slot.hash(state);
    }
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

    pub fn offset_bytes(&self) -> usize {
        if let Buffer_Handle_Inner::Non_Empty(h) = &self.inner {
            h.slot.start
        } else {
            0
        }
    }
}

impl Buffer_Allocator {
    pub fn new(buf_type: GLenum, id: Buffer_Allocator_Id) -> Self {
        debug_assert!(buf_type == gl::ARRAY_BUFFER, "Currently we are not really supporting allocs other than ARRAY_BUFFER. If we need to, the VAO should be made optional");

        Self {
            buckets: vec![],
            buf_type,
            id,
            #[cfg(debug_assertions)]
            cur_allocated: HashSet::default(),
            #[cfg(debug_assertions)]
            cur_allocated_bytes: 0,
            #[cfg(debug_assertions)]
            high_water_mark: 0,
            #[cfg(debug_assertions)]
            is_destroyed: false,
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

        #[cfg(debug_assertions)]
        {
            self.is_destroyed = true;
        }
    }

    pub fn dealloc_all(&mut self) {
        for bucket in &mut self.buckets {
            reset_bucket(bucket);
        }
        #[cfg(debug_assertions)]
        {
            self.cur_allocated.clear();
            self.cur_allocated_bytes = 0;
        }
    }

    #[must_use]
    pub fn allocate(&mut self, min_capacity: usize) -> Buffer_Handle {
        #[cfg(debug_assertions)]
        {
            debug_assert!(
                !self.is_destroyed,
                "Tried to allocate from a destroyed Buffer Allocator (with id {:?})",
                self.id
            );
        }

        if min_capacity == 0 {
            return Buffer_Handle {
                inner: Buffer_Handle_Inner::Empty,
            };
        }

        lverbose!(
            "Requesting {} from {:?} Buffer Allocator",
            format_bytes_pretty(min_capacity),
            self.id
        );

        let (bucket_idx, slot) = if let Some((bucket_idx, free_slot_idx)) =
            self.find_first_bucket_with_capacity(min_capacity)
        {
            let slot =
                allocate_from_bucket(&mut self.buckets[bucket_idx], free_slot_idx, min_capacity);

            (bucket_idx, slot)
        } else {
            let new_bucket_cap = min_capacity.max(MIN_BUCKET_SIZE);
            let mut bucket = allocate_bucket(self.buf_type, new_bucket_cap);
            let slot = allocate_from_bucket(&mut bucket, 0, min_capacity);
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
            #[cfg(debug_assertions)]
            writes: RefCell::new(vec![]),
        };

        #[cfg(debug_assertions)]
        {
            self.cur_allocated.insert(h.clone());
            self.cur_allocated_bytes += min_capacity;
            self.high_water_mark = self.high_water_mark.max(self.cur_allocated_bytes);
        }

        lverbose!(
            "...high water mark is now {}. Number of buckets: {}.",
            format_bytes_pretty(self.high_water_mark),
            self.buckets.len()
        );

        Buffer_Handle {
            inner: Buffer_Handle_Inner::Non_Empty(h),
        }
    }

    pub fn deallocate(&mut self, handle: Buffer_Handle) {
        if let Buffer_Handle_Inner::Non_Empty(h) = handle.inner {
            #[cfg(debug_assertions)]
            {
                debug_assert!(self.cur_allocated.contains(&h));
                self.cur_allocated_bytes -= h.slot.len;
                self.cur_allocated.remove(&h);
            }

            deallocate_in_bucket(&mut self.buckets[h.bucket_idx as usize], h.slot);
        }
    }

    #[inline]
    pub fn update_buffer(
        &mut self,
        handle: &Buffer_Handle,
        offset: usize,
        len: usize,
        data: *const c_void,
    ) {
        trace!("buf_alloc::update_buffer");

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
                lverbose!(
                    "For capacity {}, found bucket with free list: {:?}",
                    capacity,
                    bucket.free_list
                );
                return Some((bucket_idx, free_slot_idx));
            }
        }
        None
    }
}

// Note: intentionally not Copy and Clone (in release) so we can use it sort of like a handle
#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(debug_assertions, derive(Hash, Clone))]
pub struct Bucket_Slot {
    pub start: usize,
    pub len: usize,
}

impl Bucket_Slot {
    pub fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }

    pub fn end(&self) -> usize {
        self.start + self.len
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

#[derive(Debug)]
pub struct Buffer_Allocator_Bucket {
    vao: GLuint,
    vbo: GLuint,
    buf_type: GLenum,

    pub free_list: Vec<Bucket_Slot>,
    pub capacity: usize,
}

fn allocate_bucket(buf_type: GLenum, capacity: usize) -> Buffer_Allocator_Bucket {
    trace!("buf_alloc::allocate_bucket");

    let (mut vao, mut vbo) = (0, 0);
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        check_gl_err();
        debug_assert!(vao != 0);

        gl::GenBuffers(1, &mut vbo);
        debug_assert!(vbo != 0);

        gl::BindVertexArray(vao);
        check_gl_err();

        gl::BindBuffer(buf_type, vbo);
        check_gl_err();

        gl::BufferStorage(
            buf_type,
            capacity as _,
            ptr::null(),
            gl::DYNAMIC_STORAGE_BIT,
        );
        check_gl_err();

        gl::BindVertexArray(0);
        check_gl_err();
    }

    lverbose!(
        "Buffer_Allocator: allocated new bucket with capacity {}",
        format_bytes_pretty(capacity)
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

// If the bucket has enough room to accomodate at least `cap` bytes, returns Some(free_slot_idx); else returns None.
fn bucket_has_contiguous_capacity(bucket: &Buffer_Allocator_Bucket, cap: usize) -> Option<usize> {
    bucket.free_list.iter().position(|slot| slot.len >= cap)
}

fn is_bucket_slot_free(bucket: &Buffer_Allocator_Bucket, slot: &Bucket_Slot) -> bool {
    bucket.free_list.iter().any(|s| s.contains(slot))
}

fn is_bucket_valid(bucket: &Buffer_Allocator_Bucket) -> bool {
    if bucket.free_list.is_empty() {
        return true;
    }

    if !free_list_is_sorted(&bucket.free_list) {
        debug_assert!(false, "bucket not sorted");
        return false;
    }

    if bucket.free_list[bucket.free_list.len() - 1].end() > bucket.capacity {
        debug_assert!(false, "bucket exceeding capacity");
        return false;
    }

    true
}

#[must_use]
fn allocate_from_bucket(
    bucket: &mut Buffer_Allocator_Bucket,
    free_slot_idx: usize,
    len: usize,
) -> Bucket_Slot {
    trace!("buf_alloc::allocate_from_bucket");

    lverbose!(
        "allocating {} from bucket {:?}, slot {}",
        len,
        bucket,
        free_slot_idx
    );
    let modified_bucket = &bucket.free_list[free_slot_idx];
    let slot = Bucket_Slot::new(modified_bucket.start, len);

    debug_assert!(
        len <= modified_bucket.len,
        "allocate_from_bucket: len is {} / {}",
        len,
        modified_bucket.len
    );
    debug_assert!(is_bucket_slot_free(bucket, &slot));

    let modified_bucket = &mut bucket.free_list[free_slot_idx];
    modified_bucket.len -= len;

    if modified_bucket.len == 0 {
        bucket.free_list.remove(free_slot_idx);
    } else {
        modified_bucket.start += len;
    }

    debug_assert!(is_bucket_valid(bucket));

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
    trace!("buf_alloc::deallocate_in_bucket");

    debug_assert!(!is_bucket_slot_free(bucket, &slot));

    let slot_end = slot.end();
    if let Some(insert_pos) = bucket.free_list.iter().position(|s| s.start > slot.start) {
        // Check if we can merge this to the previous one...
        let mut inserted_pos = insert_pos;
        if insert_pos > 0 && bucket.free_list[insert_pos - 1].end() == slot.start {
            bucket.free_list[insert_pos - 1].len += slot.len;
            inserted_pos -= 1;
        } else {
            // ...else just insert it after
            bucket.free_list.insert(insert_pos, slot);
        }

        // Check if we can merge this to the next one (after we placed it)
        if inserted_pos < bucket.free_list.len() - 1
            && slot_end == bucket.free_list[inserted_pos + 1].start
        {
            bucket.free_list[inserted_pos].len += bucket.free_list[inserted_pos + 1].len;
            bucket.free_list.remove(inserted_pos + 1);
        }
    } else {
        // Inserting in last place
        if bucket.free_list.is_empty() {
            bucket.free_list.push(slot);
        } else {
            let last_idx = bucket.free_list.len() - 1;
            if bucket.free_list[last_idx].end() == slot.start {
                bucket.free_list[last_idx].len += slot.len;
            } else {
                bucket.free_list.push(slot);
            }
        }
    }

    debug_assert!(is_bucket_valid(bucket));
}

#[inline]
fn write_to_bucket(
    bucket: &mut Buffer_Allocator_Bucket,
    handle: &Non_Empty_Buffer_Handle,
    offset: usize,
    len: usize,
    data: *const c_void,
) {
    trace!("buf_alloc::write_to_bucket");

    #[cfg(debug_assertions)]
    {
        if handle.allocator_id == Buffer_Allocator_Id::Array_Temporary {
            let write = Bucket_Slot { start: (handle.slot.start + offset), len };
            for w in handle.writes.borrow().iter() {
                if w.contains(&write) {
                    panic!("vao {}: overlapping writes! existing: {:?}, new: {:?}", handle.vao, w, write);
                }
            }
            handle.writes.borrow_mut().push(write);
        }
    }

    debug_assert!(!is_bucket_slot_free(bucket, &handle.slot));
    debug_assert!(len <= handle.slot.len);

    unsafe {
        gl::BindVertexArray(bucket.vao);
        check_gl_err();

        gl::BindBuffer(bucket.buf_type, bucket.vbo);
        check_gl_err();

        gl::BufferSubData(
            bucket.buf_type,
            (handle.slot.start + offset) as _,
            len as _,
            data,
        );
        check_gl_err();
    }
}

fn reset_bucket(bucket: &mut Buffer_Allocator_Bucket) {
    bucket.free_list = vec![Bucket_Slot::new(0, bucket.capacity)];
    debug_assert!(is_bucket_valid(bucket));
}

#[cfg(test)]
mod tests {
    use super::*;
    use inle_test::test_common::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_allocate_from_bucket() {
        let (_win, _glfw) = load_gl_pointers();
        assert!(gl::GenVertexArrays::is_loaded());

        let mut bucket = allocate_bucket(gl::ARRAY_BUFFER, 100);
        assert!(is_bucket_slot_free(&bucket, &Bucket_Slot::new(0, 100)));

        let slot = allocate_from_bucket(&mut bucket, 0, 50);
        assert_eq!(slot.start, 0);
        assert_eq!(slot.len, 50);
        assert!(!is_bucket_slot_free(&bucket, &slot));
        assert!(bucket_has_contiguous_capacity(&bucket, 60).is_none());
        assert!(bucket_has_contiguous_capacity(&bucket, 50).is_some());

        deallocate_in_bucket(&mut bucket, slot);
        assert!(is_bucket_slot_free(&bucket, &Bucket_Slot::new(0, 100)));
        assert_eq!(bucket_has_contiguous_capacity(&bucket, 100), Some(0));
    }

    #[test]
    #[serial]
    fn test_allocate_from_bucket_fragmented() {
        let (_win, _glfw) = load_gl_pointers();

        let mut bucket = allocate_bucket(gl::ARRAY_BUFFER, 100);
        let slot1 = allocate_from_bucket(&mut bucket, 0, 20);
        let slot2 = allocate_from_bucket(&mut bucket, 0, 10);
        let slot3 = allocate_from_bucket(&mut bucket, 0, 30);

        deallocate_in_bucket(&mut bucket, slot2);

        assert!(bucket_has_contiguous_capacity(&bucket, 60).is_none());
        assert_eq!(bucket_has_contiguous_capacity(&bucket, 20), Some(1));
        assert_eq!(bucket_has_contiguous_capacity(&bucket, 10), Some(0));

        deallocate_in_bucket(&mut bucket, slot3);
        assert_eq!(bucket_has_contiguous_capacity(&bucket, 60), Some(0));
    }

    #[test]
    #[serial]
    fn test_reset_bucket() {
        let (_win, _glfw) = load_gl_pointers();
        let mut bucket = allocate_bucket(gl::ARRAY_BUFFER, 100);

        let slot1 = allocate_from_bucket(&mut bucket, 0, 10);
        let slot2 = allocate_from_bucket(&mut bucket, 0, 20);
        let slot3 = allocate_from_bucket(&mut bucket, 0, 30);
        let slot4 = allocate_from_bucket(&mut bucket, 0, 40);

        assert_eq!(slot1.start, 0);
        assert_eq!(slot1.len, 10);
        assert_eq!(slot2.start, 10);
        assert_eq!(slot2.len, 20);
        assert_eq!(slot3.start, 30);
        assert_eq!(slot3.len, 30);
        assert_eq!(slot4.start, 60);
        assert_eq!(slot4.len, 40);

        assert!(bucket.free_list.is_empty());

        reset_bucket(&mut bucket);

        assert_eq!(bucket_has_contiguous_capacity(&bucket, 100), Some(0));
    }

    #[test]
    #[serial]
    fn allocate_from_buffer_allocators() {
        let (_win, _glfw) = load_gl_pointers();
        let mut allocators = Buffer_Allocators::default();
        let alloc = allocators.get_alloc_mut(Buffer_Allocator_Id::Array_Permanent);
        let mut alloc = alloc.borrow_mut();

        let buf = alloc.allocate(200);
        assert!(matches!(buf.inner, Buffer_Handle_Inner::Non_Empty(_)));
        assert!(alloc.cur_allocated_bytes >= 200);

        alloc.deallocate(buf);
        assert_eq!(alloc.cur_allocated_bytes, 0);

        let _b1 = alloc.allocate(100);
        let _b2 = alloc.allocate(200);
        let _b3 = alloc.allocate(300);

        assert!(alloc.cur_allocated_bytes >= 600);
        let high_water_mark = alloc.high_water_mark;

        alloc.dealloc_all();

        assert_eq!(alloc.cur_allocated_bytes, 0);
        assert!(high_water_mark == alloc.high_water_mark);

        let _b = alloc.allocate(200);
        assert!(high_water_mark == alloc.high_water_mark);
    }
}
