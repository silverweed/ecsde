use std::alloc::{alloc, dealloc, Layout};
use std::mem::{align_of, size_of};
use std::ops::{Deref, DerefMut};

#[cfg(debug_assertions)]
pub(super) type Gen_Type = u32;

/// A simple stack allocator that does not support individual deallocations.
/// Used as a per-frame temp allocator.
pub struct Temp_Allocator {
    pub(super) ptr: *mut u8,
    pub used: usize,
    pub cap: usize,

    #[cfg(debug_assertions)]
    pub gen: Gen_Type,
}

impl Drop for Temp_Allocator {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr, Layout::from_size_align(self.cap, 1).unwrap());
        }
    }
}

impl Temp_Allocator {
    pub fn with_capacity(cap: usize) -> Self {
        let ptr = unsafe { alloc(Layout::from_size_align(cap, 1).unwrap()) };
        assert!(!ptr.is_null());

        Temp_Allocator {
            cap,
            used: 0,
            ptr,
            #[cfg(debug_assertions)]
            gen: 0,
        }
    }

    /// # Safety
    /// The caller must ensure that the returned reference is not accessed after `dealloc_all` is called.
    pub unsafe fn alloc<T>(&mut self, value: T) -> Temp_Ref<T>
    where
        T: Copy,
    {
        let size = size_of::<T>();
        let align = align_of::<T>();

        let ptr = self.alloc_bytes_aligned(size, align);

        let tptr = ptr as *mut T;
        tptr.write(value);

        Temp_Ref {
            value: &mut *tptr,
            #[cfg(debug_assertions)]
            parent_allocator: self,
            #[cfg(debug_assertions)]
            gen: self.gen,
        }
    }

    /// # Safety
    /// The caller must ensure that the returned pointer is not accessed after `dealloc_all` is called.
    /// The caller must also initialize the pointer before it's accessed.
    #[must_use]
    pub unsafe fn alloc_bytes_aligned(&mut self, n_bytes: usize, align: usize) -> *mut u8 {
        let ptr = self.ptr.add(self.used);
        let offset = self.ptr.align_offset(align);

        // @Robustness: maybe reallocate rather than crashing
        assert!(self.used + offset <= self.cap - n_bytes, "Out of memory!");

        self.used += offset + n_bytes;

        let ptr = ptr.add(offset);
        ptr
    }

    /// # Safety
    /// See `alloc`
    pub unsafe fn alloc_default<T>(&mut self) -> Temp_Ref<T>
    where
        T: Default + Copy,
    {
        self.alloc(T::default())
    }

    /// # Safety
    /// The caller must not access any returned Temp_Ref after calling this function.
    pub unsafe fn dealloc_all(&mut self) {
        self.used = 0;
        #[cfg(debug_assertions)]
        {
            self.gen = self.gen.wrapping_add(1);
        }
    }
}

pub struct Temp_Ref<T> {
    value: *mut T,

    #[cfg(debug_assertions)]
    gen: Gen_Type,
    #[cfg(debug_assertions)]
    parent_allocator: *const Temp_Allocator,
}

impl<T> Deref for Temp_Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                unsafe { &*self.parent_allocator }.gen,
                self.gen,
                "Value {:?} accessed after dealloc!",
                std::any::type_name::<T>()
            );
        }
        unsafe { &*self.value }
    }
}

impl<T> DerefMut for Temp_Ref<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                unsafe { &*self.parent_allocator }.gen,
                self.gen,
                "Value {:?} accessed after dealloc!",
                std::any::type_name::<T>()
            );
        }
        unsafe { &mut *self.value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Default, PartialEq, Debug)]
    struct Test {
        pub b: u64,
        pub a: i32,
    }

    #[derive(Copy, Clone, Default, PartialEq, Debug)]
    struct Test_Big {
        pub b: u64,
        pub a: i32,
        pub c: u64,
    }

    #[test]
    fn alloc_small() {
        let mut allocator = Temp_Allocator::with_capacity(16);

        let t1 = unsafe { allocator.alloc_default::<Test>() };
        assert_eq!(*t1, Test::default());
    }

    #[test]
    fn capacity_consistency() {
        let mut allocator = Temp_Allocator::with_capacity(16);
        let used = allocator.used;

        let t1 = unsafe { allocator.alloc_default::<Test>() };
        assert_eq!(*t1, Test::default());

        assert!(allocator.used > used);

        unsafe {
            allocator.dealloc_all();
        }
        assert_eq!(allocator.used, used);
    }

    #[test]
    #[should_panic]
    fn access_after_free() {
        let mut allocator = Temp_Allocator::with_capacity(16);

        let t = Test { a: 1, b: 2 };
        let t1 = unsafe { allocator.alloc::<Test>(t) };
        assert_eq!(*t1, t);

        unsafe {
            allocator.dealloc_all();
        }

        assert_eq!(*t1, t);
    }

    #[test]
    #[should_panic]
    fn access_after_free_2() {
        let mut allocator = Temp_Allocator::with_capacity(16);

        let mut t1 = unsafe { allocator.alloc_default::<Test>() };
        assert_eq!(*t1, Test::default());

        unsafe {
            allocator.dealloc_all();
        }

        t1.a += 2;
    }

    #[test]
    fn alloc_dealloc_small() {
        let mut allocator = Temp_Allocator::with_capacity(16);

        let t1 = unsafe { allocator.alloc_default::<Test>() };
        assert_eq!(*t1, Test::default());

        unsafe {
            allocator.dealloc_all();
        }

        let t1 = unsafe { allocator.alloc_default::<Test>() };
        assert_eq!(*t1, Test::default());
    }

    #[test]
    #[should_panic]
    fn alloc_oom() {
        let mut allocator = Temp_Allocator::with_capacity(16);

        let mut t1 = unsafe { allocator.alloc_default::<Test_Big>() };
        assert_eq!(*t1, Test_Big::default());

        t1.a = 3;
        t1.b = 44;
        assert_eq!(t1.a, 3);
        assert_eq!(t1.b, 44);
    }

    #[test]
    fn alloc_many() {
        let mut allocator = Temp_Allocator::with_capacity(128);

        let mut t1 = unsafe { allocator.alloc_default::<Test>() };
        assert_eq!(*t1, Test::default());

        let t2 = unsafe { allocator.alloc_default::<Test>() };
        assert_eq!(*t1, Test::default());
        assert_eq!(*t2, *t1);

        t1.a = 3;
        t1.b = 44;
        let t3 = unsafe { allocator.alloc_default::<Test>() };
        assert_eq!(t1.a, 3);
        assert_eq!(t1.b, 44);
        assert_eq!(*t2, Test::default());
        assert_eq!(*t2, *t3);
    }
}
