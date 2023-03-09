use alloc::vec::Vec;
use spin::mutex::SpinMutex;
use lazy_static::lazy_static;

pub struct Pid(pub usize);

pub struct PidAllocator {
    start: usize,
    end: usize,
    recycled: Vec<usize>,
}

lazy_static! {
    pub static ref PID_ALLOCATOR: SpinMutex<PidAllocator> = SpinMutex::new(PidAllocator::new(1, 4096));
}

pub fn alloc_pid() -> Option<Pid> {
    PID_ALLOCATOR.lock().alloc()
}

impl Drop for Pid {
    fn drop(&mut self) {
        PID_ALLOCATOR.lock().dealloc(self.0);
    }
}

impl PidAllocator {
    pub fn new(start: usize, end: usize) -> Self {
        return Self {start: start, end: end, recycled: Vec::new()};
    }

    pub fn alloc(&mut self) -> Option<Pid> {
        if !self.recycled.is_empty() {
            return self.recycled.pop().map(|id| {Pid(id)});
        }else {
            if self.start > self.end {
                return None;
            }
            let pid = Pid(self.start);
            self.start+=1;
            return Some(pid);
        }
    }

    pub fn dealloc(&mut self, pid: usize) {
        self.recycled.push(pid);
    }
}
