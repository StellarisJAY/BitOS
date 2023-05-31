use core::sync::atomic::AtomicIsize;
use core::sync::atomic::Ordering;
use crate::syscall::proc::sys_yield;
use crate::task::tcb::TaskControlBlock;
use crate::task::scheduler::{current_tid, block_current_task, current_task};
use alloc::collections::VecDeque;
use alloc::sync::{Weak, Arc};

pub trait Mutex: Sync + Send{
    fn lock(&self);
    fn unlock(&self);
}

pub struct SpinMutex {
    holder: AtomicIsize,  // holder 记录当前持有锁的tid
}

pub struct BlockingMutex {
    holder: AtomicIsize,
    block_queue: spin::Mutex<VecDeque<Weak<TaskControlBlock>>>, // queue 记录在等待锁的线程
}

impl SpinMutex {
    pub fn new() -> Self {
        Self{holder: AtomicIsize::new(-1)}
    }
}

impl BlockingMutex {
    pub fn new() -> Self {
        Self{
            holder: AtomicIsize::new(-1),
            block_queue: spin::Mutex::new(VecDeque::new()),
        }
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
        let tid = current_tid() as isize;
        match self.holder.compare_exchange(-1, tid, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return,
            Err(_) => {
                let task = current_task();
                self.block_queue
                .lock()
                .push_back(Arc::downgrade(&task)); // 线程加入队列末尾等待
                block_current_task();
            },
        }
    }

    fn unlock(&self) {
        let tid = current_tid() as isize;
        match self.holder.compare_exchange(tid, -1, Ordering::Relaxed, Ordering::Relaxed) {
            Err(_) => return,
            Ok(_) => {
                self.block_queue
                .lock()
                .pop_front()          //公平锁，唤醒队列中的第一个线程
                .map(|task| {
                    let task = task.upgrade().unwrap();
                    task.wake_up();
                });
            }
        }
    }
}