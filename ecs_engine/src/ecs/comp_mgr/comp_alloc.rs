use std::alloc::{alloc_zeroed, dealloc, realloc, Layout};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;
use std::ptr;

// @Speed @Convenience: consider making this configurable
const INITIAL_N_ELEMS: usize = 8;

pub struct Component_Allocator {
    /// This data is filled out by contiguous "Component Wrappers" of the form:
    /// struct {
    ///     comp: T,
    ///     next: Relative_Ptr,
    ///     prev: Relative_Ptr,
    ///     _pad: [u8; _]
    /// }
    /// whose align is >= align_of::<T>().
    /// Free slots are filled like another linked list, where 'next' points to the
    /// next free slot.
    data: *mut u8,

    // Note: the following are all relative offsets from data.
    // They are in units of Comp_Wrapper<T>, so e.g. data.offset(free_head - 1) is the address of the free head.
    // Note that 0 represents NULL, and the actual offset is (x - 1)
    /// Points to the head of the free slots linked list.
    free_head: Relative_Ptr,
    /// Points to the head of the filled slots linked list.
    filled_head: Relative_Ptr,
    /// Points to the last filled slot.
    filled_tail: Relative_Ptr,

    layout: Layout,
}

impl Component_Allocator {
    pub fn new<T: Copy>() -> Self {
        debug_assert!(mem::size_of::<Comp_Wrapper<T>>() >= mem::size_of::<usize>());

        let comp_size = mem::size_of::<T>();

        assert_ne!(comp_size, 0, "Component_Allocator cannot be used with ZST!");

        let size = INITIAL_N_ELEMS * mem::size_of::<Comp_Wrapper<T>>();
        let align = mem::align_of::<Comp_Wrapper<T>>();
        debug_assert!(align.is_power_of_two());
        let layout = Layout::from_size_align(size, align).unwrap();
        // Note: safe because size is > 0.
        // Note: we need to zero the memory because that's how we know if a free slot is linked to other
        // free slots or if it's the last one.
        let data = unsafe { alloc_zeroed(layout) };

        Self {
            data,
            free_head: Relative_Ptr::with_offset(0),
            filled_head: Relative_Ptr::null(),
            filled_tail: Relative_Ptr::null(),
            layout,
        }
    }
}

impl Drop for Component_Allocator {
    fn drop(&mut self) {
        if !self.data.is_null() {
            unsafe {
                dealloc(self.data, self.layout);
            }
        }
    }
}

impl Component_Allocator {
    /// # Safety
    /// - This function does NOT check that the memory at index `idx` is actually initialized:
    /// it is the caller's responsibility to never call this function with an invalid index.
    /// - Also, T must be the same T as the one used to construct this allocator.
    pub unsafe fn get<T: Copy>(&self, idx: u32) -> &T {
        debug_assert_eq!(mem::align_of::<Comp_Wrapper<T>>(), self.layout.align());
        &(*self.data_at_idx::<T>(idx)).comp
    }

    /// # Safety
    /// See get.
    pub unsafe fn get_mut<T: Copy>(&mut self, idx: u32) -> &mut T {
        debug_assert_eq!(mem::align_of::<Comp_Wrapper<T>>(), self.layout.align());
        &mut (*self.data_at_idx::<T>(idx)).comp
    }

    /// Returns the index where the component was added and the component itself
    pub fn add<T: Copy>(&mut self, data: T) -> (u32, &mut T) {
        debug_assert_eq!(mem::align_of::<Comp_Wrapper<T>>(), self.layout.align());

        // Check if need to grow
        {
            let wrapper_size = mem::size_of::<Comp_Wrapper<T>>();
            let remaining_space_in_bytes = self.layout.size() - wrapper_size;
            let free_head_off_in_bytes = (self.free_head.offset() as usize) * wrapper_size;
            if free_head_off_in_bytes > remaining_space_in_bytes {
                self.grow::<T>();
            }
        }

        let ptr_to_add = unsafe { self.free_head.to_abs::<T>(self.data) };

        // The free slot pointed at by self.free_head contains the relative pointer to the next free slot.
        // It may be null (i.e. 0) if self.free_head is the last free slot.
        let this_free_slot = unsafe {
            let free = ptr_to_add as *const Free_Slot;
            *free
        };
        debug_assert!(
            this_free_slot.next.is_null()
                || (this_free_slot.next.offset() as usize) * mem::size_of::<Comp_Wrapper<T>>()
                    <= self.layout.size()
        );

        // Fill this slot
        unsafe {
            ptr_to_add.write(Comp_Wrapper {
                comp: data,
                next: Relative_Ptr::null(),
                prev: self.filled_tail,
            });
        }

        let new_elem_idx = self.free_head.offset();
        let new_elem_rel_ptr = Relative_Ptr::with_offset(new_elem_idx);

        // Patch prev pointer
        if !self.filled_tail.is_null() {
            unsafe {
                let prev = self.filled_tail.to_abs::<T>(self.data);
                (*prev).next = new_elem_rel_ptr;
            }
        }

        if self.filled_head.is_null() {
            self.filled_head = self.free_head;
        }
        self.filled_tail = new_elem_rel_ptr;

        // Update free_head
        self.free_head = if this_free_slot.next.is_null() {
            // We appended at the end of the list: just advance by one.
            self.free_head.incr()
        } else {
            this_free_slot.next
        };

        unsafe { (new_elem_idx, &mut (*ptr_to_add).comp) }
    }

    /// # Safety
    /// The idx-th slot must be actually occupied.
    pub unsafe fn remove<T: Copy>(&mut self, idx: u32) {
        debug_assert_eq!(mem::align_of::<Comp_Wrapper<T>>(), self.layout.align());

        let ptr_to_removed = Relative_Ptr::with_offset(idx as u32).to_abs::<T>(self.data);

        let ptr_prev = (*ptr_to_removed).prev;
        let ptr_next = (*ptr_to_removed).next;

        // Patch the previous node's `next` pointer (or set the filled_head as the removed node's next pointer)
        if !ptr_prev.is_null() {
            let prev = ptr_prev.to_abs::<T>(self.data);
            (*prev).next = ptr_next;
        } else {
            // We're removing the head
            self.filled_head = ptr_next;
        }

        // Patch the next node's `prev` pointer
        if !ptr_next.is_null() {
            let next = ptr_next.to_abs::<T>(self.data);
            (*next).prev = ptr_prev;
        } else {
            // We're removing the tail
            self.filled_tail = ptr_prev;
        }

        // Use the freed node as the new head of the free list
        let free_slot = ptr_to_removed as *mut Free_Slot;
        free_slot.write(Free_Slot {
            next: self.free_head,
        });
        // Assert no self-reference
        debug_assert_ne!(self.free_head.offset(), idx);
        self.free_head = Relative_Ptr::with_offset(idx);
    }

    /// # Safety
    /// The idx-th slot must be filled.
    #[inline(always)]
    unsafe fn data_at_idx<T: Copy>(&self, idx: u32) -> *mut Comp_Wrapper<T> {
        let ptr = Relative_Ptr::with_offset(idx).to_abs::<T>(self.data);
        debug_assert_eq!(ptr.align_offset(mem::align_of::<Comp_Wrapper<T>>()), 0);

        ptr
    }

    #[cfg(debug_assertions)]
    fn debug_print<T: Copy + Debug>(&self) {
        let mut ptr = self.data as *mut Comp_Wrapper<T>;
        println!("----");
        unsafe {
            println!("data: {:p}", self.data);
            println!(
                "free_head: {:?} ({:p})",
                self.free_head,
                self.free_head.to_abs::<T>(self.data)
            );
            println!(
                "filled_head: {:?} ({:p})",
                self.filled_head,
                self.filled_head.to_abs::<T>(self.data)
            );
            println!("---- filled ----");
            println!("{:?} [0] [{:p}]", *ptr, ptr);
            while !(*ptr).next.is_null() {
                let ptr_next = (*ptr).next;
                ptr = ptr_next.to_abs::<T>(self.data);
                let idx = ptr_next.offset();
                println!("{:?} [{}] [{:p}]", *ptr, idx, ptr);
            }

            //let mut off = self.free_head;
            //if !off.is_null() {
            //println!("---- free ----");
            //while !off.is_null()
            //&& (off.offset() as usize) * mem::size_of::<T>() < self.layout.size()
            //{
            //let ptr = off.to_abs::<T>(self.data) as *const Free_Slot;
            //let idx = off.offset();
            //println!("{:?} [{}] [{:p}]", *ptr, idx, ptr);
            //off = (*ptr).next;
            //}
            //}
        }
    }

    #[cold]
    fn grow<T: Copy>(&mut self) {
        //println!("GROW");
        debug_assert_eq!(mem::align_of::<Comp_Wrapper<T>>(), self.layout.align());

        let new_size = self.layout.size() * 2;
        unsafe {
            self.data = realloc(self.data, self.layout, new_size);
            assert!(!self.data.is_null());

            // Since there is no such thing as realloc_zeroed, we zero the memory ourselves.
            // @Audit: is this zeroing ALL and JUST the new memory?
            ptr::write_bytes(
                self.data.add(self.layout.size()),
                0,
                new_size - self.layout.size(),
            );
        }

        self.layout = Layout::from_size_align(new_size, self.layout.align()).unwrap();
    }
}

pub struct Component_Allocator_Iter<'a, T> {
    alloc: *const Component_Allocator,
    cur: Relative_Ptr,
    _pd: PhantomData<&'a T>,
}

impl<T> Component_Allocator_Iter<'_, T> {
    pub fn empty() -> Self {
        Self {
            alloc: ptr::null(),
            cur: Relative_Ptr::null(),
            _pd: PhantomData,
        }
    }
}

impl<'a, T: 'a + Copy> Iterator for Component_Allocator_Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(!self.alloc.is_null());
        let alloc = unsafe { &*self.alloc };

        if self.cur.is_null() || (self.cur.offset() as usize) >= alloc.layout.size() {
            return None;
        }

        let data = unsafe { self.cur.to_abs::<T>(alloc.data) };
        debug_assert!(!data.is_null());

        let ref_to_comp = unsafe { &(*data).comp };
        self.cur = unsafe { (*data).next };

        Some(ref_to_comp)
    }
}

pub struct Component_Allocator_Iter_Mut<'a, T> {
    alloc: *mut Component_Allocator,
    cur: Relative_Ptr,
    _pd: PhantomData<&'a mut T>,
}

impl<T> Component_Allocator_Iter_Mut<'_, T> {
    pub fn empty() -> Self {
        Self {
            alloc: ptr::null_mut(),
            cur: Relative_Ptr::null(),
            _pd: PhantomData,
        }
    }
}

impl<'a, T: 'a + Copy> Iterator for Component_Allocator_Iter_Mut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(!self.alloc.is_null());
        let alloc = unsafe { &mut *self.alloc };

        if self.cur.is_null() || (self.cur.offset() as usize) >= alloc.layout.size() {
            return None;
        }

        let data = unsafe { self.cur.to_abs::<T>(alloc.data) };
        debug_assert!(!data.is_null());

        let ref_to_comp = unsafe { &mut (*data).comp };
        self.cur = unsafe { (*data).next };

        Some(ref_to_comp)
    }
}

impl Component_Allocator {
    pub fn iter<T>(&self) -> Component_Allocator_Iter<'_, T> {
        Component_Allocator_Iter {
            alloc: self as *const _,
            cur: self.filled_head,
            _pd: PhantomData,
        }
    }

    pub fn iter_mut<T>(&mut self) -> Component_Allocator_Iter_Mut<'_, T> {
        let filled_head = self.filled_head;
        Component_Allocator_Iter_Mut {
            alloc: self as *mut _,
            cur: filled_head,
            _pd: PhantomData,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct Relative_Ptr(u32);

impl Debug for Relative_Ptr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_null() {
            write!(f, "Relative_Ptr(null)")
        } else {
            write!(f, "Relative_Ptr(off = {})", self.0 - 1)
        }
    }
}

impl Relative_Ptr {
    pub fn null() -> Self {
        Self(0)
    }

    pub fn with_offset(off: u32) -> Self {
        Self(off + 1)
    }

    pub fn is_null(self) -> bool {
        self.0 == 0
    }

    pub fn offset(self) -> u32 {
        debug_assert_ne!(self.0, 0);
        self.0 - 1
    }

    #[must_use]
    pub fn incr(self) -> Self {
        Self(self.0 + 1)
    }

    /// # Safety
    /// If the pointer has an invalid offset from base, it's UB.
    pub unsafe fn to_abs<T: Copy>(self, base: *mut u8) -> *mut Comp_Wrapper<T> {
        if self.0 == 0 {
            ptr::null_mut()
        } else {
            (base as *mut Comp_Wrapper<T>).add(self.0 as usize - 1)
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
struct Comp_Wrapper<T: Copy> {
    pub comp: T,
    pub next: Relative_Ptr,
    pub prev: Relative_Ptr,
}

#[derive(Copy, Clone, Debug)]
struct Free_Slot {
    pub next: Relative_Ptr,
}

impl<T> Debug for Comp_Wrapper<T>
where
    T: Debug + Copy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Comp_Wrapper {{ comp: {:?}, next: {:?} }}",
            self.comp, self.next
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Debug)]
    struct C_Test {
        foo: u64,
        bar: f32,
    }

    #[test]
    fn grow() {
        let mut alloc = Component_Allocator::new::<C_Test>();
        alloc.grow::<C_Test>();
    }

    #[test]
    fn allocate() {
        let mut alloc = Component_Allocator::new::<C_Test>();

        // Add A
        let (idx, a) = alloc.add(C_Test { foo: 42, bar: 84. });
        assert_eq!(a.foo, 42);
        assert_eq!(a.bar, 84.);
        alloc.debug_print::<C_Test>();

        let a = unsafe { alloc.get::<C_Test>(idx) };
        assert_eq!(a.foo, 42);
        assert_eq!(a.bar, 84.);

        let a = unsafe { alloc.get_mut::<C_Test>(idx) };
        a.foo = 11;
        a.bar = 57.;
        assert_eq!(a.foo, 11);
        assert_eq!(a.bar, 57.);

        let a = unsafe { alloc.get::<C_Test>(idx) };
        assert_eq!(a.foo, 11);
        assert_eq!(a.bar, 57.);

        // Add B
        let (idxb, b) = alloc.add(C_Test { foo: 42, bar: 84. });
        assert_eq!(b.foo, 42);
        assert_eq!(b.bar, 84.);
        alloc.debug_print::<C_Test>();

        let a = unsafe { alloc.get::<C_Test>(idx) };
        assert_eq!(a.foo, 11);
        assert_eq!(a.bar, 57.);

        let a = unsafe { alloc.get_mut::<C_Test>(idx) };
        a.foo = 22;
        a.bar = 32.;
        assert_eq!(a.foo, 22);
        assert_eq!(a.bar, 32.);

        let b = unsafe { alloc.get::<C_Test>(idxb) };
        assert_eq!(b.foo, 42);
        assert_eq!(b.bar, 84.);

        let b = unsafe { alloc.get_mut::<C_Test>(idxb) };
        b.foo = 11;
        b.bar = 57.;
        assert_eq!(b.foo, 11);
        assert_eq!(b.bar, 57.);

        let b = unsafe { alloc.get::<C_Test>(idxb) };
        assert_eq!(b.foo, 11);
        assert_eq!(b.bar, 57.);

        for _ in 0..50 {
            alloc.add(C_Test { foo: 0, bar: 1. });
        }

        let (_, c) = alloc.add(C_Test {
            foo: 12345,
            bar: 54321.,
        });
        assert_eq!(c.foo, 12345);
        assert_eq!(c.bar, 54321.);

        let a = unsafe { alloc.get::<C_Test>(idx) };
        assert_eq!(a.foo, 22);
        assert_eq!(a.bar, 32.);

        let b = unsafe { alloc.get::<C_Test>(idxb) };
        assert_eq!(b.foo, 11);
        assert_eq!(b.bar, 57.);
    }

    #[test]
    fn remove_head() {
        let mut alloc = Component_Allocator::new::<C_Test>();

        let get_head = |alloc: &Component_Allocator| unsafe {
            (*alloc.filled_head.to_abs::<C_Test>(alloc.data)).comp
        };
        let get_tail = |alloc: &Component_Allocator| unsafe {
            (*alloc.filled_tail.to_abs::<C_Test>(alloc.data)).comp
        };

        let (i, _) = alloc.add(C_Test { foo: 0, bar: -4. });
        let (_, _) = alloc.add(C_Test { foo: 55, bar: -6.5 });

        let hd = get_head(&alloc);
        assert_eq!(hd.foo, 0);
        assert_eq!(hd.bar, -4.);

        let tl = get_tail(&alloc);
        assert_eq!(tl.foo, 55);
        assert_eq!(tl.bar, -6.5);

        unsafe {
            alloc.remove::<C_Test>(i);
        }

        let hd = get_head(&alloc);
        assert_eq!(hd.foo, 55);
        assert_eq!(hd.bar, -6.5);

        let tl = get_tail(&alloc);
        assert_eq!(tl.foo, 55);
        assert_eq!(tl.bar, -6.5);

        let (_, _) = alloc.add(C_Test { foo: 8, bar: -8. });

        let hd = get_head(&alloc);
        assert_eq!(hd.foo, 55);
        assert_eq!(hd.bar, -6.5);

        let tl = get_tail(&alloc);
        assert_eq!(tl.foo, 8);
        assert_eq!(tl.bar, -8.);
    }

    #[test]
    fn deallocate() {
        let mut alloc = Component_Allocator::new::<C_Test>();

        let (i, _) = alloc.add(C_Test { foo: 1, bar: 1. });
        unsafe {
            alloc.remove::<C_Test>(i);
        }
        let (idx, _) = alloc.add(C_Test { foo: 42, bar: 64. });

        unsafe {
            alloc.remove::<C_Test>(idx);
        }

        let (_, b) = alloc.add(C_Test {
            foo: 122,
            bar: 233.,
        });
        assert_eq!(b.foo, 122);
        assert_eq!(b.bar, 233.);
        let mut v = vec![];
        alloc.debug_print::<C_Test>();
        for _ in 0..10 {
            let (idx, _) = alloc.add(C_Test { foo: 1, bar: -2. });
            v.push(idx);
            alloc.debug_print::<C_Test>();
        }
        unsafe {
            alloc.remove::<C_Test>(3);
        }
        alloc.debug_print::<C_Test>();
        alloc.add(C_Test { foo: 33, bar: 33. });
        alloc.debug_print::<C_Test>();

        for idx in v {
            unsafe {
                alloc.remove::<C_Test>(idx);
                alloc.debug_print::<C_Test>();
            }
        }
    }
}
