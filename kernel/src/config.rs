pub const CPUS: usize = 4;
pub const PHYS_MEM_LIMIT: usize = 0x8000000000;
pub const KERNEL_HEAP_SIZE: usize = 0x00100000;
pub const PAGE_SIZE: usize = 4096;

pub const USER_STACK_SIZE: usize = 2048 * 1024;
pub const KERNEL_STACK_SIZE: usize = 8192;
pub const TRAMPOLINE: usize = PHYS_MEM_LIMIT - PAGE_SIZE + 1;