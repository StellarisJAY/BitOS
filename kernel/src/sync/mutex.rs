use core::sync::atomic::AtomicIsize;
use core::sync::atomic::Ordering;
use crate::syscall::proc::sys_yield;
use crate::task::scheduler::current_tid;
pub trait Mutex: Sync + Send{
    fn lock(&self);
    fn unlock(&self);
}

pub struct SpinMutex {
    holder: AtomicIsize,  // holder 记录当前持有锁的tid
}

pub struct BlockingMutex {
    
}

impl SpinMutex {
    pub fn new() -> Self {
        Self{holder: AtomicIsize::new(-1)}
    }
}

impl BlockingMutex {
    pub fn new() -> Self {
        Self{}
    }
}

impl Mutex for SpinMutex {
    fn lock(&self) {
        let tid = current_tid() as isize;
        // 自旋CAS，将空闲锁的-1改为自己的tid
        while self.holder.compare_exchange(-1, tid, Ordering::Relaxed, Ordering::Relaxed).is_err() {
            sys_yield();
        }
    }

    fn unlock(&self) {
        let tid = current_tid() as isize;
        // CAS释放锁，将自己的tid改成-1
        self.holder.compare_exchange(tid, -1, Ordering::Relaxed, Ordering::Relaxed);
    }
}

impl Mutex for BlockingMutex {
    fn lock(&self) {
        
    }
    fn unlock(&self) {
        
    }
}