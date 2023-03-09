pub mod address;
pub mod page_table;
pub mod allocator;
pub mod heap;
pub mod memory_set;
pub mod kernel;
pub mod app;

pub fn init() {
    heap::init();
    kernel!("kernel heap initialized");
    allocator::init();
    kernel!("phys frame allocator initialized");
    memory_set::MemorySet::init_kernel();
    kernel::kernel_map_test();
    kernel!("kernel memory mapped and initialized");
}