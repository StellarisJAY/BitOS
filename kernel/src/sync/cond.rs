use super::mutex::Mutex;
use crate::task::scheduler::current_task;
use crate::task::tcb::TaskControlBlock;
use alloc::collections::VecDeque;
use alloc::sync::{Arc, Weak};

pub struct Cond {
    wait_queue: spin::Mutex<VecDeque<Weak<TaskControlBlock>>>,
}

impl Cond {
    pub fn new() -> Self {
        Self {
            wait_queue: spin::Mutex::new(VecDeque::new()),
        }
    }

    pub fn wait(&self, mutex: Arc<dyn Mutex>) {
        mutex.unlock();
        let task = current_task();
        self.wait_queue.lock().push_back(Arc::downgrade(&task));
    }

    pub fn signal(&self) {
        self.wait_queue.lock().pop_front().map(|task| {
            task.upgrade().unwrap().wake_up();
        });
    }
}
