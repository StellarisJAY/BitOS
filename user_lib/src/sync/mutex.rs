use crate::{mutex_create, mutex_lock, mutex_unlock};

pub struct Mutex (isize);

impl Mutex {
    pub fn new(blocking: bool) -> Self {
        let id = mutex_create(blocking);
        Self(id)
    }
    
    pub fn lock(&self) {
        mutex_lock(self.0);
    }
    
    pub fn unlock(&self) {
        mutex_unlock(self.0);
    }

    pub fn id(&self) -> isize {
        self.0
    }
}