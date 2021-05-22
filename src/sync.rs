use core::{cell::Cell, ops::Deref};

/// A thin wrapper around `core::cell::Cell` that implements the `Sync` trait.
///
/// Since this type implements `Sync`, it can be used in static variables.
///
/// ## Safety
/// This type assumes that there is only a single thread in the system. It also
/// assumes that it is not accessed from interrupt handlers. If these assumptions are
/// not met, undefined behavior might occur.
pub struct SyncCell<T>(Cell<T>);

impl<T> SyncCell<T> {
    pub const fn new(value: T) -> Self {
        Self(Cell::new(value))
    }
}

impl<T> Deref for SyncCell<T> {
    type Target = Cell<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl<T: Sync> Sync for SyncCell<T> {}
