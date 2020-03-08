use std::alloc::{alloc, dealloc, Layout};
use std::mem::{align_of, size_of};
use std::ops::{Deref, DerefMut};

pub struct Temp_Allocator {
    ptr: *mut u8,
    off: usize,
    cap: usize,
    #[cfg(debug_assertions)]
    gen: u64,
}

pub struct Temp_Ref<T> {
    value: *mut T,

    #[cfg(debug_assertions)]
    gen: u64,
    #[cfg(debug_assertions)]
    parent_allocator: *const Temp_Allocator,
}

impl<T> Deref for Temp_Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        debug_assert_eq!(
            unsafe { &*self.parent_allocator }.gen,
            self.gen,
            "Value {:?} accessed after dealloc!",
            std::any::type_name::<T>()
        );
        unsafe { &*self.value }
    }
}

impl<T> DerefMut for Temp_Ref<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        debug_assert_eq!(
            unsafe { &*self.parent_allocator }.gen,
            self.gen,
            "Value {:?} accessed after dealloc!",
            std::any::type_name::<T>()
        );
        unsafe { &mut *self.value }
    }
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
            off: 0,
            ptr,
            #[cfg(debug_assertions)]
            gen: 0,
        }
    }

    /// # Safety
    /// The caller must ensure that the returned reference is not accessed after `dealloc_all` is called.
    pub unsafe fn alloc<T>(&mut self) -> Temp_Ref<T>
    where
        T: Default + Copy,
    {
        let size = size_of::<T>();
        let align = align_of::<T>();

        let ptr = self.ptr.add(self.off);
        let offset = self.ptr.align_offset(align);

        // @Robustness: maybe reallocate rather than crashing
        assert!(self.off + offset <= self.cap - size, "Out of memory!");

        let ptr = ptr.add(offset);
        let tptr = ptr as *mut T;

        self.off += offset + size;

        tptr.write(T::default());
        Temp_Ref {
            value: &mut *tptr,
            #[cfg(debug_assertions)]
            parent_allocator: self,
            #[cfg(debug_assertions)]
            gen: self.gen,
        }
    }

    /// # Safety
    /// The caller must not access any returned Temp_Ref after calling this function.
    pub unsafe fn dealloc_all(&mut self) {
        self.off = 0;
        self.gen += 1;
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

        let t1 = unsafe { allocator.alloc::<Test>() };
        assert_eq!(*t1, Test::default());
    }

    #[test]
    #[should_panic]
    fn access_after_free() {
        let mut allocator = Temp_Allocator::with_capacity(16);

        let t1 = unsafe { allocator.alloc::<Test>() };
        assert_eq!(*t1, Test::default());

        unsafe {
            allocator.dealloc_all();
        }

        assert_eq!(*t1, Test::default());
    }

    #[test]
    #[should_panic]
    fn access_after_free_2() {
        let mut allocator = Temp_Allocator::with_capacity(16);

        let mut t1 = unsafe { allocator.alloc::<Test>() };
        assert_eq!(*t1, Test::default());

        unsafe {
            allocator.dealloc_all();
        }

        t1.a += 2;
    }

    #[test]
    fn alloc_dealloc_small() {
        let mut allocator = Temp_Allocator::with_capacity(16);

        let t1 = unsafe { allocator.alloc::<Test>() };
        assert_eq!(*t1, Test::default());

        unsafe {
            allocator.dealloc_all();
        }

        let t1 = unsafe { allocator.alloc::<Test>() };
        assert_eq!(*t1, Test::default());
    }

    #[test]
    #[should_panic]
    fn alloc_oom() {
        let mut allocator = Temp_Allocator::with_capacity(16);

        let mut t1 = unsafe { allocator.alloc::<Test_Big>() };
        assert_eq!(*t1, Test_Big::default());

        t1.a = 3;
        t1.b = 44;
        assert_eq!(t1.a, 3);
        assert_eq!(t1.b, 44);
    }

    #[test]
    fn alloc_many() {
        let mut allocator = Temp_Allocator::with_capacity(128);

        let mut t1 = unsafe { allocator.alloc::<Test>() };
        assert_eq!(*t1, Test::default());

        let t2 = unsafe { allocator.alloc::<Test>() };
        assert_eq!(*t1, Test::default());
        assert_eq!(*t2, *t1);

        t1.a = 3;
        t1.b = 44;
        let t3 = unsafe { allocator.alloc::<Test>() };
        assert_eq!(t1.a, 3);
        assert_eq!(t1.b, 44);
        assert_eq!(*t2, Test::default());
        assert_eq!(*t2, *t3);
    }
}
