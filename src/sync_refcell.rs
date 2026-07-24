use core::cell::{Ref, RefCell, RefMut};

/// `RefCell` wrapper that implements `Sync` — safe only in single-core
/// `no_std` environments with no threading.
pub struct SyncRefCell<T> {
    inner: RefCell<T>,
}

// SAFETY: This kernel runs single-core; no concurrent access is possible.
unsafe impl<T> Sync for SyncRefCell<T> {}

impl<T> SyncRefCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.inner.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
