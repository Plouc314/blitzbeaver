use std::{ops::Deref, sync::Arc};

/// Smart pointer for shared values with unchecked exclusive access.
///
/// This is useful when synchronization is handled by some external mechanism,
/// rendering the overhead of a lock (mutex) unnecessary.
pub struct ExclusiveShared<T>(Arc<T>)
where
    T: Send + Sync;

impl<T> ExclusiveShared<T>
where
    T: Send + Sync,
{
    pub fn new(value: T) -> Self {
        Self(Arc::new(value))
    }

    /// Returns a mutable reference to the value.
    ///
    /// This method is safe if no other thread as access to the value.
    pub fn exclusive(&mut self) -> &mut T {
        unsafe { &mut *(Arc::as_ptr(&self.0) as *mut T) }
    }
}

impl<T> Deref for ExclusiveShared<T>
where
    T: Send + Sync,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Clone for ExclusiveShared<T>
where
    T: Send + Sync,
{
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
