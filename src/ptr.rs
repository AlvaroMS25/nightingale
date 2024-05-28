use std::ops::Deref;
use std::ptr::NonNull;

/// A wrapper around a raw pointer, this is intended to be used with items that have a known lifetime,
/// this way we avoid having a refcount. However, this type requires manual drop while ensuring no
/// more instances are alive.
#[derive(Copy)]
pub struct SharedPtr<T: ?Sized> {
    inner: NonNull<T>
}

impl<T> SharedPtr<T> {
    pub fn new(item: T) -> Self {
        let inner = Box::into_raw(Box::new(item));

        unsafe {
            Self::from_ptr_unchecked(inner)
        }
    }

    pub unsafe fn from_ptr_unchecked(ptr: *mut T) -> Self {
        Self {
            inner: unsafe { NonNull::new_unchecked(ptr) }
        }
    }
}

impl<T: ?Sized> SharedPtr<T> {
    /// Frees the underlying data.
    ///
    /// *SAFETY*: This function must be run only when a single pointer remains pointing to the
    /// underlying data
    pub unsafe fn drop_data(&self) {
        drop(Box::from_raw(self.inner.as_ptr()))
    }
}

impl<T: ?Sized> Deref for SharedPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Pointer can't be null
        unsafe { self.inner.as_ref() }
    }
}

impl<T: ?Sized> From<Box<T>> for SharedPtr<T> {
    fn from(value: Box<T>) -> Self {
        let ptr = Box::into_raw(value);
        SharedPtr {
            inner: unsafe { NonNull::new_unchecked(ptr) }
        }
    }
}

impl<T: ?Sized> Clone for SharedPtr<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner
        }
    }
}

unsafe impl<T: Send + Sync + ?Sized> Send for SharedPtr<T> {}
unsafe impl<T: Send + Sync + ?Sized> Sync for SharedPtr<T> {}
