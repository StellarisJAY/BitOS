use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::mutex::SpinMutex;

// TID分配器，每个进程有一个独立的分配器
pub struct TidAllocator {
    pub start: usize,
    pub end: usize,
    pub recycled: Vec<usize>,
}

impl TidAllocator {
    pub fn new(start: usize, end: usize) -> Self {
        return Self {
            start: start,
            end: end,
            recycled: Vec::new(),
        };
    }

    pub fn clone(&self) -> Self {
        TidAllocator {
            start: self.start,
            end: self.end,
            recycled: self.recycled.clone(),
        }
    }

    pub fn alloc(&mut self) -> Option<usize> {
        if !self.recycled.is_empty() {
            return self.recycled.pop();
        } else {
            if self.start > self.end {
                return None;
            }
            let tid = self.start;
            self.start += 1;
            return Some(tid);
        }
    }

    pub fn dealloc(&mut self, tid: usize) {
        self.recycled.push(tid);
    }
}
