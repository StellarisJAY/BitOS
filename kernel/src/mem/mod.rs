pub mod address;
pub mod page_table;
pub mod allocator;
pub mod heap;
pub mod memory_set;

pub fn init() {
    heap::init();
    kernel!("kernel heap initialized");
    allocator::init();
    kernel!("phys frame allocator initialized");
}