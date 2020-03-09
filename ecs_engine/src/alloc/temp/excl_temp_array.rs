use super::temp_alloc::Temp_Allocator;
use std::ops::{Index, IndexMut};

#[cfg(debug_assertions)]
use super::temp_alloc::Gen_Type;

/// Like Temp_Array, but it "takes ownership" of the allocator until it's dropped.
/// This means that:
/// a) it can grow indefinitely (up to the allocator's capacity)
/// b) it gives back the allocated memory on drop
pub struct Exclusive_Temp_Array<'a, T>
where
    T: Copy,
{
    ptr: *mut T,
    n_elems: usize,
    parent_allocator: &'a mut Temp_Allocator,

    #[cfg(debug_assertions)]
    gen: Gen_Type,
}

impl<T> Drop for Exclusive_Temp_Array<'_, T>
where
    T: Copy,
{
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            assert_eq!(
                self.parent_allocator.gen, self.gen,
                "Exclusive_Temp_Array accessed after free!"
            );
        }
	// @WaitForStable here we'll want to use offset_from().
	let diff = self.parent_allocator.ptr as usize + self.parent_allocator.used - self.ptr as usize;
        self.parent_allocator.ptr = self.ptr as *mut u8;
	self.parent_allocator.used -= diff;
    }
}

/// Creates a growable array that allocates from the given Temp_Allocator.
/// Cannot outlive the allocator, and its elements MUST NOT be accessed after calling
/// allocator.dealloc_all().
pub fn excl_temp_array<'a, T>(allocator: &'a mut Temp_Allocator) -> Exclusive_Temp_Array<'a, T>
where
    T: Copy + Default,
{
    let offset = allocator.ptr.align_offset(std::mem::align_of::<T>());
    #[cfg(debug_assertions)]
    let gen = allocator.gen;
    Exclusive_Temp_Array {
        ptr: unsafe { allocator.ptr.add(offset) } as *mut T,
        n_elems: 0,
        parent_allocator: allocator,
        #[cfg(debug_assertions)]
        gen,
    }
}

impl<T> Exclusive_Temp_Array<'_, T>
where
    T: Copy,
{
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
                *ptr
            };
            self.n_elems -= 1;
            Some(elem)
        } else {
            None
        }
    }

    pub fn iter(&self) -> Exclusive_Temp_Array_Iterator<'_, T> {
        Exclusive_Temp_Array_Iterator { array: self, i: 0 }
    }
}

impl<T: Copy> Index<usize> for Exclusive_Temp_Array<'_, T> {
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

impl<T: Copy> IndexMut<usize> for Exclusive_Temp_Array<'_, T> {
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

pub struct Exclusive_Temp_Array_Iterator<'a, T>
where
    T: Copy,
{
    array: &'a Exclusive_Temp_Array<'a, T>,
    i: usize,
}

impl<'a, T> Iterator for Exclusive_Temp_Array_Iterator<'a, T>
where
    T: Copy,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.array.n_elems {
            let item = &self.array[self.i];
            self.i += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<'a, T> IntoIterator for &'a Exclusive_Temp_Array<'a, T>
where
    T: Copy,
{
    type Item = &'a T;
    type IntoIter = Exclusive_Temp_Array_Iterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

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
}
