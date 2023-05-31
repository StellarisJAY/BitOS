use crate::{cond_create, cond_signal, cond_wait};
use crate::mutex_lock;
use super::mutex::Mutex;

pub struct Cond {
    cond_id: isize,
    mutex_id: isize,
}

impl Cond {
    pub fn new(mutex: &Mutex) -> Self {
        Self { cond_id: cond_create(), mutex_id: mutex.id() }
    }
    
    pub fn wait(&self) {
        match cond_wait(self.cond_id, self.mutex_id) {
            -1 => panic!("condvar doesn't exist"),
            -2 => panic!("mutex doesn't exist"),
            _ => {
                // wait被唤醒后需要重新获得锁
                mutex_lock(self.mutex_id);
            }
        }
    }
    
    pub fn signal(&self) {
        if cond_signal(self.cond_id) == -1 {
            panic!("condvar doesn't exist");
        }
    }
}