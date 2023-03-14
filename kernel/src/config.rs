pub const CPUS: usize = 1;
pub const PHYS_MEM_LIMIT: usize = 0x8000000000;
pub const KERNEL_HEAP_SIZE: usize = 0x00100000;
pub const PAGE_SIZE: usize = 4096;

pub const USER_STACK_SIZE: usize = 2048 * 1024;
pub const KERNEL_STACK_SIZE: usize = 8192;
pub const MAX_VA: usize = (1<<39) - 1;
pub const TRAMPOLINE: usize = MAX_VA - PAGE_SIZE;
// 守护页，在栈的底端加上一个不被页表映射的页，栈溢出时会触发Pagefault，方便捕获栈溢出异常
pub const GUARD_PAGE: usize = PAGE_SIZE;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
// 用户进程的内核栈位置
pub fn kernel_stack_position(pid: usize) -> (usize, usize) {
    let stack_top = TRAMPOLINE - pid * (KERNEL_HEAP_SIZE + GUARD_PAGE);
    let stack_bottom = stack_top - KERNEL_HEAP_SIZE;
    return (stack_bottom, stack_top);
}