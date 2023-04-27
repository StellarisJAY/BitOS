use crate::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;

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

#[allow(unused)]
pub fn heap_test() {
    use alloc::vec::Vec;
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for i in 0..500 {
        assert_eq!(v[i], i);
    }
    drop(v);
    println!("heap_test passed!");
}
