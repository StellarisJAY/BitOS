use core::cell::{RefCell, RefMut};


pub struct SafeCell<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for SafeCell<T> {}

impl<T> SafeCell<T> {
    pub fn new(item: T) -> Self {
        Self {
            inner: RefCell::new(item),
        }
    }

    pub fn borrow_inner(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}