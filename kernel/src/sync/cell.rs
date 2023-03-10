use spin::mutex::SpinMutex;
use core::cell::{RefCell, RefMut};

pub struct SafeCell<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for SafeCell<T> {}

impl<T> SafeCell<T> {
    pub fn new(val: T) -> Self {
        return Self { inner: RefCell::new(val) };
    }
    pub fn borrow(&self) -> RefMut<'_, T> {
        return self.inner.borrow_mut();
    }
}