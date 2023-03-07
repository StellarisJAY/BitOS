use buddy_system_allocator::LockedHeap;
use crate::config::KERNEL_HEAP_SIZE;

// 必须为mut，否则会被编译器分配到rodata只读段
static mut HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::new();

pub fn init() {
    unsafe {
        let start = HEAP.as_ptr() as usize;
        ALLOCATOR.lock().init(start, KERNEL_HEAP_SIZE);
    }
}

