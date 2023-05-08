pub mod address;
pub mod allocator;
pub mod app;
pub mod heap;
pub mod kernel;
pub mod kernel_stack;
pub mod memory_set;
pub mod page_table;

pub fn init() {
    heap::init();
    kernel!("kernel heap initialized");
    allocator::init();
    kernel!("phys frame allocator initialized");
    memory_set::MemorySet::init_kernel();
}
