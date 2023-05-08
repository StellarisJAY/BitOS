use crate::config::{GUARD_PAGE, KERNEL_STACK_SIZE, TRAMPOLINE};
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::mutex::SpinMutex;
pub struct KernelStack(pub usize);

// 内核栈分配器，不再通过pid或tid标记内核栈，而是通过一个专门的内核栈id
pub struct StackIdAllocator {
    start: usize,
    end: usize,
    recycled: Vec<usize>,
}

lazy_static! {
    pub static ref KSTACK_ALLOCATOR: SpinMutex<StackIdAllocator> =
        SpinMutex::new(StackIdAllocator::new(0, 1024));
}

pub fn kernel_stack_position(id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - id * (KERNEL_STACK_SIZE + GUARD_PAGE);
    let bottom = top - KERNEL_STACK_SIZE;
    return (bottom, top);
}

pub fn alloc_kstack() -> Option<KernelStack> {
    KSTACK_ALLOCATOR.lock().alloc()
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        KSTACK_ALLOCATOR.lock().dealloc(self.0);
    }
}

impl StackIdAllocator {
    pub fn new(start: usize, end: usize) -> Self {
        return Self {
            start: start,
            end: end,
            recycled: Vec::new(),
        };
    }

    pub fn alloc(&mut self) -> Option<KernelStack> {
        if !self.recycled.is_empty() {
            return self.recycled.pop().map(|id| KernelStack(id));
        } else {
            if self.start > self.end {
                return None;
            }
            let pid = KernelStack(self.start);
            self.start += 1;
            return Some(pid);
        }
    }

    pub fn dealloc(&mut self, pid: usize) {
        self.recycled.push(pid);
    }
}
