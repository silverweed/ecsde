use super::temp_alloc::Temp_Allocator;
use std::mem::{align_of, size_of};
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::marker::PhantomData;
use crate::common::thread_safe_ptr::Thread_Safe_Ptr;

#[cfg(debug_assertions)]
use super::temp_alloc::Gen_Type;

/// A temporary array that "takes ownership" of the allocator until it's dropped.
/// This means that:
/// a) it can grow indefinitely (up to the allocator's capacity)
/// b) it gives back the allocated memory on drop
pub struct Exclusive_Temp_Array<'a, T> {
    ptr: *mut T,
    n_elems: usize,
    parent_allocator: &'a mut Temp_Allocator,
    parent_alloc_prev_used: usize,

    #[cfg(debug_assertions)]
    gen: Gen_Type,
}

impl<T> Drop for Exclusive_Temp_Array<'_, T> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                self.parent_allocator.gen, self.gen,
                "Exclusive_Temp_Array accessed after free!"
            );
        }

        if std::mem::needs_drop::<T>() {
            for i in 0..self.n_elems {
                unsafe {
                    self.ptr.add(i).drop_in_place();
                }
            }
        }

        self.parent_allocator.used = self.parent_alloc_prev_used;
    }
}

/// Creates a growable array that allocates from the given Temp_Allocator.
/// Cannot outlive the allocator, and its elements MUST NOT be accessed after calling
/// allocator.dealloc_all().
pub fn excl_temp_array<T>(allocator: &mut Temp_Allocator) -> Exclusive_Temp_Array<'_, T> {
    let ptr = unsafe { allocator.ptr.add(allocator.used) };
    let offset = ptr.align_offset(std::mem::align_of::<T>());

    let parent_alloc_prev_used = allocator.used;

    #[cfg(debug_assertions)]
    let gen = allocator.gen;
    Exclusive_Temp_Array {
        ptr: unsafe { ptr.add(offset) } as *mut T,
        n_elems: 0,
        parent_allocator: allocator,
        parent_alloc_prev_used,
        #[cfg(debug_assertions)]
        gen,
    }
}

impl<T> Exclusive_Temp_Array<'_, T> {
    pub fn as_slice(&self) -> &'_ [T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.n_elems) }
    }

    pub fn as_slice_mut(&mut self) -> &'_ mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.n_elems) }
    }

    pub fn len(&self) -> usize {
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                self.parent_allocator.gen, self.gen,
                "Exclusive_Temp_Array accessed after free!"
            );
        }
        self.n_elems
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Expands this array by `new_elems` elements (not bytes), allocating them
    /// from the temp allocator and not initializing them.
    ///
    /// # Safety
    /// It's UB to read any uninitialized element added this way.
    pub unsafe fn alloc_additional_uninit(&mut self, new_elems: usize) {
        let _ = self
            .parent_allocator
            .alloc_bytes_aligned(new_elems * size_of::<T>(), align_of::<T>());
        self.n_elems += new_elems;
    }

    pub fn push(&mut self, elem: T) {
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                self.parent_allocator.gen, self.gen,
                "Exclusive_Temp_Array accessed after free!"
            );
        }
        unsafe { self.parent_allocator.alloc(elem) };
        self.n_elems += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                self.parent_allocator.gen, self.gen,
                "Exclusive_Temp_Array accessed after free!"
            );
        }
        if self.n_elems > 0 {
            let elem = unsafe {
                let ptr = self.ptr.add(self.n_elems);
                ptr.read()
            };
            self.n_elems -= 1;
            Some(elem)
        } else {
            None
        }
    }
}

impl<T> Exclusive_Temp_Array<'_, T> {
    /// Transforms this Exclusive_Temp_Array into a read-only version of itself.
    /// This is used to relinquish the mutable borrow and allow the Temp_Allocator
    /// to be used again while keeping this data.
    ///
    /// # Safety
    /// Since the lifetime information about this data is lost in the process, the
    /// caller must ensure he does not retain this data after clearing the Temp_Allocator.
    /// The data is safe to access otherwise.
    pub unsafe fn into_read_only(self) -> Read_Only_Temp_Array<T> {
        let arr = Read_Only_Temp_Array {
            ptr: Thread_Safe_Ptr::from(self.ptr as *mut u8),
            n_elems: self.n_elems,
            _pd: PhantomData,
            #[cfg(debug_assertions)]
            parent_allocator: Thread_Safe_Ptr::from(self.parent_allocator as *const _ as *mut _),
            #[cfg(debug_assertions)]
            gen: self.gen,
        };

        std::mem::forget(self);

        arr
    }
}

impl<T> Index<usize> for Exclusive_Temp_Array<'_, T> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        assert!(idx < self.n_elems);
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                self.parent_allocator.gen, self.gen,
                "Exclusive_Temp_Array accessed after free!"
            );
        }
        unsafe { &*self.ptr.add(idx) }
    }
}

impl<T> IndexMut<usize> for Exclusive_Temp_Array<'_, T> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        assert!(idx < self.n_elems);
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                self.parent_allocator.gen, self.gen,
                "Exclusive_Temp_Array accessed after free!"
            );
        }
        unsafe { &mut *self.ptr.add(idx) }
    }
}

impl<T> Deref for Exclusive_Temp_Array<'_, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for Exclusive_Temp_Array<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<'a, T> IntoIterator for &'a Exclusive_Temp_Array<'a, T> {
    type IntoIter = std::slice::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Extend<&'a T> for Exclusive_Temp_Array<'a, T>
where
    T: 'a + Copy,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a T>,
    {
        for x in iter {
            self.push(*x);
        }
    }
}

pub struct Read_Only_Temp_Array<T> {
    ptr: Thread_Safe_Ptr<u8>,
    n_elems: usize,
    _pd: PhantomData<T>,

    #[cfg(debug_assertions)]
    parent_allocator: Thread_Safe_Ptr<Temp_Allocator>,
    #[cfg(debug_assertions)]
    gen: Gen_Type,
}

impl<T> Drop for Read_Only_Temp_Array<T> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                unsafe { (*self.parent_allocator.raw()).gen },
                self.gen,
                "Exclusive_Temp_Array accessed after free!"
            );
        }

        if std::mem::needs_drop::<T>() {
            let ptr: *mut T = self.ptr.raw_mut() as *mut _;
            for i in 0..self.n_elems {
                unsafe {
                    ptr.add(i).drop_in_place();
                }
            }
        }
    }
}

impl<T> Read_Only_Temp_Array<T> {
    pub fn as_slice(&self) -> &'_ [T] {
        unsafe { std::slice::from_raw_parts(self.ptr.raw_mut() as *mut _, self.n_elems) }
    }
}

impl<T> Index<usize> for Read_Only_Temp_Array<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        assert!(idx < self.n_elems);
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                unsafe { (*self.parent_allocator.raw()).gen },
                self.gen,
                "Exclusive_Temp_Array accessed after free!"
            );
        }
        let ptr = self.ptr.raw_mut() as *mut T;
        unsafe { &*ptr.add(idx) }
    }
}

impl<T> Deref for Read_Only_Temp_Array<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'a, T> IntoIterator for &'a Read_Only_Temp_Array<T> {
    type IntoIter = std::slice::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

unsafe impl<T> Send for Read_Only_Temp_Array<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_excl_temp_array() {
        let mut allocator = Temp_Allocator::with_capacity(100);
        let mut tmpary: Exclusive_Temp_Array<u64> = excl_temp_array(&mut allocator);
        tmpary.push(1);
        tmpary.push(2);
        tmpary.push(3);
        tmpary.push(4);
        assert_eq!(tmpary[0], 1);
        assert_eq!(tmpary[1], 2);
        assert_eq!(tmpary[2], 3);
        assert_eq!(tmpary[3], 4);
    }

    #[test]
    #[should_panic]
    fn excl_temp_array_oob_access() {
        let mut allocator = Temp_Allocator::with_capacity(100);
        let tmpary: Exclusive_Temp_Array<u64> = excl_temp_array(&mut allocator);
        assert_eq!(tmpary[0], 1);
    }

    #[test]
    #[should_panic]
    fn excl_temp_array_access_after_free() {
        let mut allocator = Temp_Allocator::with_capacity(100);
        let tmpary: Exclusive_Temp_Array<u64> = excl_temp_array(&mut allocator);
        assert_eq!(tmpary[0], 1);
    }

    #[test]
    #[should_panic]
    fn excl_temp_array_push_over_alloc_cap() {
        let mut allocator = Temp_Allocator::with_capacity(10);
        let mut tmpary: Exclusive_Temp_Array<u64> = excl_temp_array(&mut allocator);
        tmpary.push(1);
        tmpary.push(2);
    }

    #[test]
    fn excl_temp_array_return_cap() {
        let mut allocator = Temp_Allocator::with_capacity(100);
        {
            let mut tmpary: Exclusive_Temp_Array<u64> = excl_temp_array(&mut allocator);
            tmpary.push(1);
            tmpary.push(2);
        }
        assert_eq!(allocator.cap - allocator.used, 100);
    }

    #[test]
    fn excl_temp_array_into_read_only() {
        let mut allocator = Temp_Allocator::with_capacity(100);
        let mut tmpary: Exclusive_Temp_Array<u64> = excl_temp_array(&mut allocator);
        tmpary.push(1);
        tmpary.push(2);

        let roary = unsafe { tmpary.into_read_only() };

        let mut tmpary: Exclusive_Temp_Array<u64> = excl_temp_array(&mut allocator);
        tmpary.push(3);
        tmpary.push(4);

        assert_eq!(roary[0], 1);
        assert_eq!(roary[1], 2);
        assert_eq!(tmpary[0], 3);
        assert_eq!(tmpary[1], 4);
    }
}
