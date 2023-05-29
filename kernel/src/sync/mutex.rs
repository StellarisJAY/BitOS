use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use crate::syscall::proc::sys_yield;

pub trait Mutex: Sync + Send{
    fn lock(&self);
    fn unlock(&self);
}

pub struct SpinMutex {
    locked: AtomicBool,
}

pub struct BlockingMutex {
    
}

impl SpinMutex {
    pub fn new() -> Self {
        Self{locked: AtomicBool::new(false)}
    }
}

impl BlockingMutex {
    pub fn new() -> Self {
        Self{}
    }
}

impl Mutex for SpinMutex {
    fn lock(&self) {
        while self.locked.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed).is_err() {
            sys_yield();
        }
    }
    fn unlock(&self) {
        self.locked.store(false, Ordering::Relaxed);
    }
}

impl Mutex for BlockingMutex {
    fn lock(&self) {
        
    }
    fn unlock(&self) {
        
    }
}