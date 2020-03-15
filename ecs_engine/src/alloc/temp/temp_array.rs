use super::temp_alloc::Temp_Allocator;
use std::marker::PhantomData;
use std::mem::{align_of, size_of};
use std::ops::{Deref, DerefMut, Index, IndexMut};

#[cfg(debug_assertions)]
use super::temp_alloc::Gen_Type;

pub struct Temp_Array<'a, T> {
    ptr: *mut T,
    n_elems: usize,
    cap: usize,
    _pd: PhantomData<&'a T>,

    #[cfg(debug_assertions)]
    parent_allocator: &'a Temp_Allocator,
    #[cfg(debug_assertions)]
    gen: Gen_Type,
}

/// Creates a growable array that allocates from the given Temp_Allocator.
/// Cannot outlive the allocator, and its elements MUST NOT be accessed after calling
/// allocator.dealloc_all().
pub fn temp_array<T>(allocator: &mut Temp_Allocator, capacity: usize) -> Temp_Array<'_, T> {
    let ptr = unsafe { allocator.alloc_bytes_aligned(capacity * size_of::<T>(), align_of::<T>()) }
        as *mut T;

    Temp_Array {
        ptr,
        n_elems: 0,
        cap: capacity,
        _pd: PhantomData,
        #[cfg(debug_assertions)]
        parent_allocator: allocator,
        #[cfg(debug_assertions)]
        gen: allocator.gen,
    }
}

impl<T> Temp_Array<'_, T> {
    pub fn len(&self) -> usize {
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                self.parent_allocator.gen, self.gen,
                "Temp_Array accessed after free!"
            );
        }
        self.n_elems
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, elem: T) {
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                self.parent_allocator.gen, self.gen,
                "Temp_Array accessed after free!"
            );
        }
        assert!(
            self.n_elems < self.cap,
            "Temp_Array is at full capacity! {}/{}",
            self.n_elems,
            self.cap
        );

        unsafe {
            let ptr = self.ptr.add(self.n_elems);
            ptr.write(elem);
        }

        self.n_elems += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                self.parent_allocator.gen, self.gen,
                "Temp_Array accessed after free!"
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

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.n_elems) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.n_elems) }
    }
}

impl<T> Index<usize> for Temp_Array<'_, T> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        assert!(idx < self.n_elems);
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                self.parent_allocator.gen, self.gen,
                "Temp_Array accessed after free!"
            );
        }
        unsafe { &*self.ptr.add(idx) }
    }
}

impl<T> IndexMut<usize> for Temp_Array<'_, T> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        assert!(idx < self.n_elems);
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                self.parent_allocator.gen, self.gen,
                "Temp_Array accessed after free!"
            );
        }
        unsafe { &mut *self.ptr.add(idx) }
    }
}

impl<T> Deref for Temp_Array<'_, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for Temp_Array<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_array() {
        let mut allocator = Temp_Allocator::with_capacity(100);
        let mut tmpary: Temp_Array<u64> = temp_array(&mut allocator, 10);
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
    fn temp_array_oob_access() {
        let mut allocator = Temp_Allocator::with_capacity(100);
        let tmpary: Temp_Array<u64> = temp_array(&mut allocator, 10);
        assert_eq!(tmpary[0], 1);
    }

    #[test]
    #[should_panic]
    fn temp_array_access_after_free() {
        let mut allocator = Temp_Allocator::with_capacity(100);
        let tmpary: Temp_Array<u64> = temp_array(&mut allocator, 10);
        assert_eq!(tmpary[0], 1);
    }

    #[test]
    #[should_panic]
    fn temp_array_push_over_cap() {
        let mut allocator = Temp_Allocator::with_capacity(100);
        let mut tmpary: Temp_Array<u64> = temp_array(&mut allocator, 3);
        tmpary.push(1);
        tmpary.push(2);
        tmpary.push(3);
        tmpary.push(4);
    }

    #[test]
    #[should_panic]
    fn temp_array_push_over_alloc_cap() {
        let mut allocator = Temp_Allocator::with_capacity(10);
        let mut tmpary: Temp_Array<u64> = temp_array(&mut allocator, 10);
        tmpary.push(1);
        tmpary.push(2);
    }
}
