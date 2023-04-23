pub const CPUS: usize = 1;
pub const PHYS_MEM_LIMIT: usize = 2 << 30;
pub const KERNEL_HEAP_SIZE: usize = 0x00100000;
pub const PAGE_SIZE: usize = 4096;

// 每个进程的用户态栈大小：
pub const USER_STACK_SIZE: usize = 8 * 1024;
// 每个进程的内核栈大小：4KiB
pub const KERNEL_STACK_SIZE: usize = 4 * 1024;
// 虚拟地址最大值
pub const MAX_VA: usize = 4<<30;
pub const TRAMPOLINE: usize = MAX_VA - PAGE_SIZE;
// 守护页，在栈的底端加上一个不被页表映射的页，栈溢出时会触发Pagefault，方便捕获栈溢出异常
pub const GUARD_PAGE: usize = PAGE_SIZE;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

// 最大pid值
pub const MAX_PID: usize = 2<<15;
// 内核栈区域底部地址
pub const KERNEL_STACK_BOTTOM: usize = TRAMPOLINE - MAX_PID * (KERNEL_STACK_SIZE + GUARD_PAGE);

// 用户进程的内核栈位置
pub fn kernel_stack_position(pid: usize) -> (usize, usize) {
    let stack_top = TRAMPOLINE - pid * (KERNEL_STACK_SIZE + GUARD_PAGE);
    let stack_bottom = stack_top - KERNEL_STACK_SIZE;
    return (stack_bottom, stack_top);
}