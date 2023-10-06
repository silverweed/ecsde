/// Use with care! This class is inherently unsafe and it's the
/// user's responsibility to use it properly.
#[derive(Debug)]
pub struct Thread_Safe_Ptr<T>(*mut T);

impl<T> Clone for Thread_Safe_Ptr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Thread_Safe_Ptr<T> {}

unsafe impl<T> Send for Thread_Safe_Ptr<T> {}
unsafe impl<T> Sync for Thread_Safe_Ptr<T> {}

impl<T> Thread_Safe_Ptr<T> {
    pub fn raw(self) -> *const T {
        self.0
    }

    pub fn raw_mut(self) -> *mut T {
        self.0
    }
}

impl<T> From<Thread_Safe_Ptr<T>> for *mut T {
    fn from(ssp: Thread_Safe_Ptr<T>) -> Self {
        ssp.raw_mut()
    }
}

impl<T> From<*mut T> for Thread_Safe_Ptr<T> {
    fn from(ptr: *mut T) -> Self {
        Self(ptr)
    }
}
